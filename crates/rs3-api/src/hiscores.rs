use crate::client::Rs3Client;
use crate::error::Rs3ApiError;
use crate::models::{
    HiscoreActivity, HiscoreActivityEntry, PlayerStats, Skill, SkillEntry,
};

const HISCORES_URL: &str = "https://secure.runescape.com/m=hiscore/index_lite.ws";
const SKILL_COUNT: usize = 30;
const ACTIVITY_COUNT: usize = 31;

impl Rs3Client {
    /// Fetches and parses hiscores data for a player.
    pub async fn get_player_stats(&self, player_name: &str) -> Result<PlayerStats, Rs3ApiError> {
        let url = format!("{}?player={}", HISCORES_URL, player_name);
        let response = self.http.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Rs3ApiError::PlayerNotFound(player_name.to_string()));
        }

        let body = response.text().await?;
        parse_hiscores(&body, player_name)
    }
}

/// Parses the raw hiscores lite CSV into typed structs.
///
/// The response is whitespace-separated entries. Each entry is comma-separated:
/// - Skills (first 30): rank,level,xp
/// - Activities (next 31): rank,count
fn parse_hiscores(body: &str, player_name: &str) -> Result<PlayerStats, Rs3ApiError> {
    let entries: Vec<&str> = body.split_whitespace().collect();
    let expected = SKILL_COUNT + ACTIVITY_COUNT;

    if entries.len() < expected {
        return Err(Rs3ApiError::Parse(format!(
            "Expected at least {} entries for player '{}', got {}",
            expected, player_name, entries.len()
        )));
    }

    let mut skills = Vec::with_capacity(SKILL_COUNT);
    for (i, entry) in entries[..SKILL_COUNT].iter().enumerate() {
        let parts: Vec<&str> = entry.split(',').collect();
        if parts.len() != 3 {
            return Err(Rs3ApiError::Parse(format!(
                "Skill entry {} has {} parts, expected 3: '{}'",
                i, parts.len(), entry
            )));
        }

        let skill = Skill::from_index(i).ok_or_else(|| {
            Rs3ApiError::Parse(format!("Unknown skill at index {}", i))
        })?;

        let rank = parts[0].parse::<i64>().map_err(|e| {
            Rs3ApiError::Parse(format!("Bad rank in skill entry {}: {}", i, e))
        })?;
        let level = parts[1].parse::<u16>().map_err(|e| {
            Rs3ApiError::Parse(format!("Bad level in skill entry {}: {}", i, e))
        })?;
        let xp = parts[2].parse::<u64>().map_err(|e| {
            Rs3ApiError::Parse(format!("Bad XP in skill entry {}: {}", i, e))
        })?;

        skills.push(SkillEntry { skill, rank, level, xp });
    }

    let mut activities = Vec::with_capacity(ACTIVITY_COUNT);
    for (i, entry) in entries[SKILL_COUNT..SKILL_COUNT + ACTIVITY_COUNT].iter().enumerate() {
        let parts: Vec<&str> = entry.split(',').collect();
        if parts.len() != 2 {
            return Err(Rs3ApiError::Parse(format!(
                "Activity entry {} has {} parts, expected 2: '{}'",
                i, parts.len(), entry
            )));
        }

        let activity = HiscoreActivity::from_index(i).ok_or_else(|| {
            Rs3ApiError::Parse(format!("Unknown activity at index {}", i))
        })?;

        let rank = parts[0].parse::<i64>().map_err(|e| {
            Rs3ApiError::Parse(format!("Bad rank in activity entry {}: {}", i, e))
        })?;
        let count = parts[1].parse::<i64>().map_err(|e| {
            Rs3ApiError::Parse(format!("Bad count in activity entry {}: {}", i, e))
        })?;

        activities.push(HiscoreActivityEntry { activity, rank, count });
    }

    Ok(PlayerStats { skills, activities })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real hiscores response for pastafartian
    const SAMPLE_RESPONSE: &str = "104122,2993,834489804 89086,106,27247418 83898,99,48414581 111807,104,21415713 105615,99,54459706 176653,100,15467202 129162,99,15211727 73559,113,53353383 161865,99,13873484 124297,101,16905138 214259,99,13178588 175729,99,14437064 124466,101,16432288 120449,101,16062189 127806,101,16507577 177917,100,15198198 90548,120,104393746 89263,99,20818390 89796,108,33804205 118136,110,39085569 117930,107,29298563 132698,99,13978157 127073,99,14723368 110235,99,15972587 165603,99,13459177 128736,104,23407997 144472,99,13979017 83260,120,98278597 112764,110,41577545 155256,99,13548630 -1,0 -1,0 -1,0 -1,0 -1,1 42422,1212 18514,1607 67971,701 35724,1731 -1,1672 -1,0 -1,1000 -1,50 -1,0 -1,0 -1,0 -1,0 -1,0 -1,0 -1,0 -1,0 18870,107 -1,0 -1,0 58659,15790 284838,1 164354,2 201492,3 88565,29 60076,9 -1,0";

    #[test]
    fn parses_skills_correctly() {
        let stats = parse_hiscores(SAMPLE_RESPONSE, "pastafartian").unwrap();

        assert_eq!(stats.skills.len(), 30);

        // Overall
        let overall = &stats.skills[0];
        assert_eq!(overall.skill, Skill::Overall);
        assert_eq!(overall.rank, 104122);
        assert_eq!(overall.level, 2993);
        assert_eq!(overall.xp, 834489804);

        // Attack
        let attack = &stats.skills[1];
        assert_eq!(attack.skill, Skill::Attack);
        assert_eq!(attack.level, 106);

        // Necromancy (last skill)
        let necro = &stats.skills[29];
        assert_eq!(necro.skill, Skill::Necromancy);
        assert_eq!(necro.level, 99);
    }

    #[test]
    fn parses_activities_correctly() {
        let stats = parse_hiscores(SAMPLE_RESPONSE, "pastafartian").unwrap();

        assert_eq!(stats.activities.len(), 31);

        // Bounty Hunter (unranked)
        let bh = &stats.activities[0];
        assert_eq!(bh.activity, HiscoreActivity::BountyHunter);
        assert_eq!(bh.rank, -1);
        assert_eq!(bh.count, 0);

        // BA Attacker (ranked)
        let ba_atk = &stats.activities[5];
        assert_eq!(ba_atk.activity, HiscoreActivity::BaAttacker);
        assert_eq!(ba_atk.rank, 42422);
        assert_eq!(ba_atk.count, 1212);

        // RuneScore
        let runescore = &stats.activities[24];
        assert_eq!(runescore.activity, HiscoreActivity::RuneScore);
        assert_eq!(runescore.count, 15790);

        // Clue Elite
        let clue_elite = &stats.activities[28];
        assert_eq!(clue_elite.activity, HiscoreActivity::ClueElite);
        assert_eq!(clue_elite.count, 29);
    }

    #[test]
    fn rejects_short_response() {
        let result = parse_hiscores("1,2,3 4,5,6", "test");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Rs3ApiError::Parse(_)));
    }

    #[test]
    fn rejects_malformed_skill_entry() {
        // First entry has only 2 values instead of 3
        let mut entries: Vec<&str> = vec!["1,2"];
        let good_skills: Vec<&str> = vec!["1,2,3"; 29];
        entries.extend(good_skills);
        let good_activities: Vec<&str> = vec!["1,2"; 31];
        entries.extend(good_activities);
        let body = entries.join(" ");

        let result = parse_hiscores(&body, "test");
        assert!(result.is_err());
    }
}
