use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────
// Boss
// ─────────────────────────────────────────────

/// A tracked boss stored in the `bosses` collection.
/// The `slug` field serves as the document `_id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boss {
    /// Used as MongoDB `_id`, e.g. "arch-glacor"
    #[serde(rename = "_id")]
    pub slug: String,
    pub name: String,
    pub kills: u32,
    pub initial_kills: u32,
    pub total_wealth: u64,
    pub tracked_since: DateTime<Utc>,
}

// ─────────────────────────────────────────────
// Drop
// ─────────────────────────────────────────────

/// A significant drop stored in the `drops` collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drop {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub boss_slug: String,
    pub item_name: String,
    pub gp_value: u64,
    pub received_at: DateTime<Utc>,
    pub logged_at: DateTime<Utc>,
}

// ─────────────────────────────────────────────
// Goal
// ─────────────────────────────────────────────

/// A goal stored in the `goals` collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub goal_type: GoalType,
    pub category: Option<String>,
    pub boss_slug: Option<String>,
    pub skill_id: Option<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GoalType {
    Numeric { current: u64, target: u64 },
    Checkbox { completed: bool },
    Checklist { items: Vec<ChecklistItem> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub name: String,
    pub completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ChecklistItem>>,
}

// ─────────────────────────────────────────────
// Tracked Activity
// ─────────────────────────────────────────────

/// A hiscores activity tracked over time (clue scrolls, RuneScore, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedActivity {
    /// The activity name, used as `_id` (e.g. "ClueHard")
    #[serde(rename = "_id")]
    pub activity: String,
    pub count: i64,
    pub initial_count: i64,
    pub tracked_since: DateTime<Utc>,
}

// ─────────────────────────────────────────────
// Settings
// ─────────────────────────────────────────────

/// App settings stored as a single document in the `settings` collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "_id")]
    pub id: String, // always "default"
    pub player_name: String,
    pub activity_cursor: Option<ActivityCursor>,
}

/// Tracks where we left off in the RuneMetrics activity feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityCursor {
    pub last_activity_date: String,
    pub last_activity_text: String,
}
