use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct PlayerProfile {
    pub name: String,
    pub combat_level: u16,
    pub total_skill: u16,
    pub total_xp: u64,
    pub quests_complete: u16,
    pub logged_in: bool,
}

/// Fetches the player's hiscores and RuneMetrics profile.
#[tauri::command]
pub async fn get_player_profile(
    state: State<'_, AppState>,
    player_name: String,
) -> Result<PlayerProfile, String> {
    let profile = state
        .rs3
        .get_runemetrics_profile(&player_name, 0)
        .await
        .map_err(|e| e.to_string())?;

    Ok(PlayerProfile {
        name: profile.name,
        combat_level: profile.combat_level,
        total_skill: profile.total_skill,
        total_xp: profile.total_xp,
        quests_complete: profile.quests_complete,
        logged_in: profile.logged_in,
    })
}
