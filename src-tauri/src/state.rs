use common_db::DbClient;
use rs3_api::Rs3Client;

/// Shared application state managed by Tauri.
/// Created once at startup, accessible from all commands via `State<AppState>`.
pub struct AppState {
    pub rs3: Rs3Client,
    pub db: DbClient,
}
