use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// Boss data as returned to the frontend.
#[derive(Serialize)]
pub struct BossResponse {
    pub slug: String,
    pub name: String,
    pub kills: u32,
    pub initial_kills: u32,
    pub total_wealth: u64,
    pub tracked_since: String,
}

/// Drop data as returned to the frontend.
#[derive(Serialize)]
pub struct DropResponse {
    pub id: String,
    pub boss_slug: String,
    pub item_name: String,
    pub gp_value: u64,
    pub received_at: String,
    pub logged_at: String,
}

/// Input for creating a new boss.
#[derive(Deserialize)]
pub struct CreateBossInput {
    pub slug: String,
    pub name: String,
    pub initial_kills: u32,
}

/// Input for logging a drop.
#[derive(Deserialize)]
pub struct LogDropInput {
    pub boss_slug: String,
    pub item_name: String,
    pub gp_value: u64,
    pub received_at: Option<String>,
}

impl From<common_db::models::Boss> for BossResponse {
    fn from(boss: common_db::models::Boss) -> Self {
        Self {
            slug: boss.slug,
            name: boss.name,
            kills: boss.kills,
            initial_kills: boss.initial_kills,
            total_wealth: boss.total_wealth,
            tracked_since: boss.tracked_since.to_rfc3339(),
        }
    }
}

impl From<common_db::models::Drop> for DropResponse {
    fn from(drop: common_db::models::Drop) -> Self {
        Self {
            id: drop.id.map(|id| id.to_hex()).unwrap_or_default(),
            boss_slug: drop.boss_slug,
            item_name: drop.item_name,
            gp_value: drop.gp_value,
            received_at: drop.received_at.to_rfc3339(),
            logged_at: drop.logged_at.to_rfc3339(),
        }
    }
}

/// Returns all tracked bosses.
#[tauri::command]
pub async fn get_bosses(state: State<'_, AppState>) -> Result<Vec<BossResponse>, String> {
    let bosses = state.db.get_all_bosses().await.map_err(|e| e.to_string())?;
    Ok(bosses.into_iter().map(BossResponse::from).collect())
}

/// Returns a single boss by slug.
#[tauri::command]
pub async fn get_boss(state: State<'_, AppState>, slug: String) -> Result<BossResponse, String> {
    let boss = state.db.get_boss(&slug).await.map_err(|e| e.to_string())?;
    Ok(BossResponse::from(boss))
}

/// Creates a new tracked boss.
#[tauri::command]
pub async fn create_boss(
    state: State<'_, AppState>,
    input: CreateBossInput,
) -> Result<BossResponse, String> {
    let boss = state
        .db
        .create_boss(&input.slug, &input.name, input.initial_kills)
        .await
        .map_err(|e| e.to_string())?;
    Ok(BossResponse::from(boss))
}

/// Adds misc loot GP to a boss's wealth.
#[tauri::command]
pub async fn add_boss_wealth(
    state: State<'_, AppState>,
    slug: String,
    gp: u64,
) -> Result<(), String> {
    state
        .db
        .add_boss_wealth(&slug, gp)
        .await
        .map_err(|e| e.to_string())
}

/// Deletes a tracked boss.
#[tauri::command]
pub async fn delete_boss(state: State<'_, AppState>, slug: String) -> Result<(), String> {
    state
        .db
        .delete_boss(&slug)
        .await
        .map_err(|e| e.to_string())
}

/// Returns all significant drops for a boss.
#[tauri::command]
pub async fn get_drops(
    state: State<'_, AppState>,
    boss_slug: String,
) -> Result<Vec<DropResponse>, String> {
    let drops = state
        .db
        .get_drops_for_boss(&boss_slug)
        .await
        .map_err(|e| e.to_string())?;
    Ok(drops.into_iter().map(DropResponse::from).collect())
}

/// Logs a significant drop. Automatically adds GP to boss wealth.
#[tauri::command]
pub async fn log_drop(
    state: State<'_, AppState>,
    input: LogDropInput,
) -> Result<DropResponse, String> {
    let received_at = input
        .received_at
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let drop = state
        .db
        .log_drop(&input.boss_slug, &input.item_name, input.gp_value, received_at)
        .await
        .map_err(|e| e.to_string())?;
    Ok(DropResponse::from(drop))
}

/// Polls RuneMetrics for new boss kills and increments kill counts.
#[tauri::command]
pub async fn sync_boss_kills(
    state: State<'_, AppState>,
    player_name: String,
) -> Result<SyncResult, String> {
    // Get the current cursor from settings
    let settings = state.db.get_settings().await.map_err(|e| e.to_string())?;
    let cursor = settings.as_ref().and_then(|s| s.activity_cursor.as_ref());

    // Convert common_db cursor to rs3_api cursor
    let api_cursor = cursor.map(|c| rs3_api::models::ActivityCursor {
        last_activity_date: c.last_activity_date.clone(),
        last_activity_text: c.last_activity_text.clone(),
    });

    // Poll for new kills
    let poll = state
        .rs3
        .get_new_boss_kills(&player_name, api_cursor.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    // Increment kill counts for each boss
    let mut kills_added: Vec<KillUpdate> = Vec::new();
    for event in &poll.kill_events {
        // Try to increment — if the boss isn't tracked, skip it
        let slug = boss_name_to_slug(&event.boss_name);
        if state.db.increment_boss_kills(&slug, event.kill_count).await.is_ok() {
            kills_added.push(KillUpdate {
                boss_slug: slug,
                boss_name: event.boss_name.clone(),
                kills: event.kill_count,
            });
        }
    }

    // Save the updated cursor
    if let Some(new_cursor) = &poll.new_cursor {
        let db_cursor = common_db::models::ActivityCursor {
            last_activity_date: new_cursor.last_activity_date.clone(),
            last_activity_text: new_cursor.last_activity_text.clone(),
        };
        // Only update if settings exist
        if settings.is_some() {
            let _ = state.db.update_activity_cursor(&db_cursor).await;
        }
    }

    Ok(SyncResult {
        kills_added,
        cursor_gap: poll.cursor_gap,
    })
}

#[derive(Serialize)]
pub struct SyncResult {
    pub kills_added: Vec<KillUpdate>,
    pub cursor_gap: bool,
}

#[derive(Serialize)]
pub struct KillUpdate {
    pub boss_slug: String,
    pub boss_name: String,
    pub kills: u32,
}

/// Converts a boss name from RuneMetrics (e.g. "Arch-Glacor") to a slug (e.g. "arch-glacor").
fn boss_name_to_slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}
