mod commands;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize shared state
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match init_state().await {
                    Ok(state) => {
                        handle.manage(state);
                        log::info!("App state initialized successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to initialize app state: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Player
            commands::player::get_player_profile,
            // Bosses
            commands::bosses::get_bosses,
            commands::bosses::get_boss,
            commands::bosses::create_boss,
            commands::bosses::add_boss_wealth,
            commands::bosses::delete_boss,
            commands::bosses::get_drops,
            commands::bosses::log_drop,
            commands::bosses::sync_boss_kills,
            // Prices
            commands::prices::get_item_price,
            commands::prices::get_item_detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn init_state() -> Result<AppState, Box<dyn std::error::Error>> {
    let rs3 = rs3_api::Rs3Client::new()?;

    // TODO: Read connection string from config/settings UI
    // For now, use environment variable
    let mongo_uri = std::env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let db_name = std::env::var("MONGODB_DB")
        .unwrap_or_else(|_| "gielinor".to_string());

    let db = common_db::DbClient::new(&mongo_uri, &db_name).await?;
    log::info!("Connected to MongoDB database '{}'", db_name);

    Ok(AppState { rs3, db })
}
