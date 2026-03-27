use serde::Deserialize;

use crate::client::Rs3Client;
use crate::error::Rs3ApiError;
use crate::models::{Activity, ActivityCursor, BossKillEvent, RuneMetricsProfile, RuneMetricsSkill};

const PROFILE_URL: &str = "https://apps.runescape.com/runemetrics/profile/profile";

// ─────────────────────────────────────────────
// Raw deserialization structs
//
// These map directly to the JSON field names from
// the API. We convert them into our clean public
// models so the rest of the app doesn't deal with
// quirks like XP * 10 or "loggedIn" as a string.
// ─────────────────────────────────────────────

#[derive(Deserialize)]
struct RawRuneMetricsResponse {
    name: String,
    combatlevel: u16,
    totalskill: u16,
    totalxp: u64,
    rank: Option<String>,
    #[serde(rename = "loggedIn")]
    logged_in: String,
    skillvalues: Vec<RawSkillValue>,
    activities: Vec<RawActivity>,
    questscomplete: u16,
    questsstarted: u16,
    questsnotstarted: u16,
}

#[derive(Deserialize)]
struct RawSkillValue {
    id: u8,
    level: u16,
    xp: u64,
    rank: i64,
}

#[derive(Deserialize)]
struct RawActivity {
    date: String,
    details: String,
    text: String,
}

impl Rs3Client {
    /// Fetches the RuneMetrics profile for a player.
    pub async fn get_runemetrics_profile(
        &self,
        player_name: &str,
        activity_count: u8,
    ) -> Result<RuneMetricsProfile, Rs3ApiError> {
        let url = format!(
            "{}?user={}&activities={}",
            PROFILE_URL, player_name, activity_count
        );
        let response = self.http.get(&url).send().await?;
        let body = response.text().await?;

        // RuneMetrics returns HTTP 200 with an error field for private/missing profiles
        if body.contains("\"error\"") {
            if body.contains("PROFILE_PRIVATE") {
                return Err(Rs3ApiError::PrivateProfile(player_name.to_string()));
            }
            return Err(Rs3ApiError::PlayerNotFound(player_name.to_string()));
        }

        let raw: RawRuneMetricsResponse = serde_json::from_str(&body).map_err(|e| {
            Rs3ApiError::Parse(format!("Failed to parse RuneMetrics response: {}", e))
        })?;

        Ok(raw.into())
    }

    /// Fetches the RuneMetrics activity feed and returns new boss kill events
    /// since the given cursor position. Also returns the updated cursor.
    ///
    /// If `cursor` is None, all activities are treated as new (first run).
    /// If the cursor's entry is no longer in the feed, all activities are
    /// returned with `cursor_gap: true` to indicate a potential gap.
    pub async fn get_new_boss_kills(
        &self,
        player_name: &str,
        cursor: Option<&ActivityCursor>,
    ) -> Result<BossKillPollResult, Rs3ApiError> {
        let profile = self.get_runemetrics_profile(player_name, 20).await?;

        let (new_activities, cursor_gap) = match cursor {
            None => (profile.activities.clone(), false),
            Some(cursor) => {
                let cursor_pos = profile.activities.iter().position(|a| {
                    a.date == cursor.last_activity_date && a.text == cursor.last_activity_text
                });

                match cursor_pos {
                    Some(pos) => (profile.activities[..pos].to_vec(), false),
                    None => (profile.activities.clone(), true),
                }
            }
        };

        let kill_events: Vec<BossKillEvent> = new_activities
            .iter()
            .filter_map(|a| parse_boss_kill(a))
            .collect();

        let new_cursor = profile.activities.first().map(|a| ActivityCursor {
            last_activity_date: a.date.clone(),
            last_activity_text: a.text.clone(),
        });

        Ok(BossKillPollResult {
            kill_events,
            new_cursor,
            cursor_gap,
        })
    }
}

/// Result of polling RuneMetrics for new boss kills.
#[derive(Debug, Clone)]
pub struct BossKillPollResult {
    /// New boss kill events since the last poll.
    pub kill_events: Vec<BossKillEvent>,
    /// Updated cursor to save for the next poll.
    pub new_cursor: Option<ActivityCursor>,
    /// True if the previous cursor was not found in the feed,
    /// meaning some activities may have been missed.
    pub cursor_gap: bool,
}

// ─────────────────────────────────────────────
// Raw -> Clean model conversion
// ─────────────────────────────────────────────

impl From<RawRuneMetricsResponse> for RuneMetricsProfile {
    fn from(raw: RawRuneMetricsResponse) -> Self {
        let skills = raw
            .skillvalues
            .into_iter()
            .map(|sv| RuneMetricsSkill {
                id: sv.id,
                level: sv.level,
                xp: sv.xp / 10, // API returns XP * 10
                rank: sv.rank,
            })
            .collect();

        let activities = raw
            .activities
            .into_iter()
            .map(|a| Activity {
                date: a.date,
                text: a.text,
                details: a.details,
            })
            .collect();

        Self {
            name: raw.name,
            combat_level: raw.combatlevel,
            total_skill: raw.totalskill,
            total_xp: raw.totalxp,
            rank: raw.rank,
            logged_in: raw.logged_in == "true",
            skills,
            activities,
            quests_complete: raw.questscomplete,
            quests_started: raw.questsstarted,
            quests_not_started: raw.questsnotstarted,
        }
    }
}

// ─────────────────────────────────────────────
// Boss kill parsing
//
// Activity text patterns from real data:
//   "I killed 6 Kerapacs."          — multiple kills
//   "I killed  an Arch-Glacor."     — single kill (double space + article)
//   "I killed  a TzKal-Zuk."        — single kill (double space + article)
//   "I killed 11 Arch-Glacors."     — multiple kills, pluralized name
// ─────────────────────────────────────────────

/// Attempts to parse a boss kill event from a RuneMetrics activity.
/// Returns None if the activity is not a boss kill.
fn parse_boss_kill(activity: &Activity) -> Option<BossKillEvent> {
    let text = &activity.text;

    if !text.starts_with("I killed") {
        return None;
    }

    let remainder = text
        .trim_start_matches("I killed")
        .trim_end_matches('.')
        .trim();

    if remainder.is_empty() {
        return None;
    }

    let (count, boss_name) = if remainder.starts_with(char::is_numeric) {
        // "6 Kerapacs" -> count=6, name="Kerapacs"
        let first_space = remainder.find(' ')?;
        let count_str = &remainder[..first_space];
        let name = remainder[first_space..].trim();
        let count = count_str.parse::<u32>().ok()?;
        (count, name.to_string())
    } else {
        // "an Arch-Glacor" / "a TzKal-Zuk" -> count=1, strip article
        let name = remainder
            .trim_start_matches("an ")
            .trim_start_matches("a ");
        (1, name.to_string())
    };

    // Depluralize for multiple kills: "Kerapacs" -> "Kerapac"
    let boss_name = if count > 1 {
        boss_name
            .strip_suffix('s')
            .unwrap_or(&boss_name)
            .to_string()
    } else {
        boss_name
    };

    Some(BossKillEvent {
        boss_name,
        kill_count: count,
        date: activity.date.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_activity(text: &str) -> Activity {
        Activity {
            date: "24-Mar-2026 11:53".to_string(),
            text: text.to_string(),
            details: String::new(),
        }
    }

    // ── Boss kill parsing ──

    #[test]
    fn parses_multiple_kills() {
        let event = parse_boss_kill(&make_activity("I killed 6 Kerapacs.")).unwrap();
        assert_eq!(event.kill_count, 6);
        assert_eq!(event.boss_name, "Kerapac");
    }

    #[test]
    fn parses_single_kill_with_an() {
        let event = parse_boss_kill(&make_activity("I killed  an Arch-Glacor.")).unwrap();
        assert_eq!(event.kill_count, 1);
        assert_eq!(event.boss_name, "Arch-Glacor");
    }

    #[test]
    fn parses_single_kill_with_a() {
        let event = parse_boss_kill(&make_activity("I killed  a TzKal-Zuk.")).unwrap();
        assert_eq!(event.kill_count, 1);
        assert_eq!(event.boss_name, "TzKal-Zuk");
    }

    #[test]
    fn parses_plural_boss_name() {
        let event = parse_boss_kill(&make_activity("I killed 11 Arch-Glacors.")).unwrap();
        assert_eq!(event.kill_count, 11);
        assert_eq!(event.boss_name, "Arch-Glacor");
    }

    #[test]
    fn ignores_non_kill_activities() {
        assert!(parse_boss_kill(&make_activity("48000000XP in Archaeology")).is_none());
        assert!(parse_boss_kill(&make_activity("Quest complete: Violet is Blue")).is_none());
        assert!(parse_boss_kill(&make_activity("Levelled up Archaeology.")).is_none());
        assert!(parse_boss_kill(&make_activity("450 Quest Points obtained")).is_none());
    }

    // ── Cursor logic ──

    #[test]
    fn cursor_finds_correct_split_point() {
        let activities = vec![
            Activity {
                date: "27-Mar-2026 01:39".to_string(),
                text: "I killed 6 Kerapacs.".to_string(),
                details: String::new(),
            },
            Activity {
                date: "25-Mar-2026 22:57".to_string(),
                text: "I killed 11 Arch-Glacors.".to_string(),
                details: String::new(),
            },
            Activity {
                date: "24-Mar-2026 23:05".to_string(),
                text: "I killed 4 Arch-Glacors.".to_string(),
                details: String::new(),
            },
        ];

        let cursor = ActivityCursor {
            last_activity_date: "25-Mar-2026 22:57".to_string(),
            last_activity_text: "I killed 11 Arch-Glacors.".to_string(),
        };

        let cursor_pos = activities.iter().position(|a| {
            a.date == cursor.last_activity_date && a.text == cursor.last_activity_text
        });

        assert_eq!(cursor_pos, Some(1));
        let new_activities = &activities[..cursor_pos.unwrap()];
        assert_eq!(new_activities.len(), 1);
        assert!(new_activities[0].text.contains("Kerapac"));
    }

    #[test]
    fn missing_cursor_signals_gap() {
        let activities = vec![
            make_activity("I killed 6 Kerapacs."),
            make_activity("I killed 11 Arch-Glacors."),
        ];

        let cursor = ActivityCursor {
            last_activity_date: "01-Jan-2020 00:00".to_string(),
            last_activity_text: "old activity".to_string(),
        };

        let cursor_pos = activities.iter().position(|a| {
            a.date == cursor.last_activity_date && a.text == cursor.last_activity_text
        });

        assert!(cursor_pos.is_none());
    }

    // ── Profile deserialization ──

    #[test]
    fn deserializes_raw_profile() {
        let json = r#"{
            "name": "Pastafartian",
            "combatlevel": 148,
            "totalskill": 3001,
            "totalxp": 868388777,
            "rank": "101,645",
            "loggedIn": "false",
            "magic": 15213342,
            "melee": 458401679,
            "ranged": 56277951,
            "questscomplete": 325,
            "questsstarted": 5,
            "questsnotstarted": 30,
            "skillvalues": [
                { "level": 120, "xp": 1045412137, "rank": 76784, "id": 26 }
            ],
            "activities": [
                {
                    "date": "27-Mar-2026 01:39",
                    "details": "I killed 6 Kerapacs, wielders of two elder artifacts.",
                    "text": "I killed 6 Kerapacs."
                }
            ]
        }"#;

        let raw: RawRuneMetricsResponse = serde_json::from_str(json).unwrap();
        let profile: RuneMetricsProfile = raw.into();

        assert_eq!(profile.name, "Pastafartian");
        assert_eq!(profile.combat_level, 148);
        assert_eq!(profile.total_skill, 3001);
        assert!(!profile.logged_in);
        assert_eq!(profile.quests_complete, 325);
        assert_eq!(profile.quests_started, 5);
        assert_eq!(profile.quests_not_started, 30);

        // XP should be divided by 10
        assert_eq!(profile.skills[0].xp, 104541213);
        assert_eq!(profile.skills[0].id, 26);

        assert_eq!(profile.activities.len(), 1);
        assert!(profile.activities[0].text.contains("Kerapac"));
    }
}
