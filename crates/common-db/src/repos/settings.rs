use mongodb::bson::doc;
use mongodb::options::ReplaceOptions;
use mongodb::Collection;

use crate::client::DbClient;
use crate::error::DbError;
use crate::models::{ActivityCursor, Settings};

const COLLECTION: &str = "settings";
const SETTINGS_ID: &str = "default";

impl DbClient {
    fn settings(&self) -> Collection<Settings> {
        self.database().collection(COLLECTION)
    }

    /// Gets the app settings. Returns None if no settings document exists yet.
    pub async fn get_settings(&self) -> Result<Option<Settings>, DbError> {
        Ok(self
            .settings()
            .find_one(doc! { "_id": SETTINGS_ID })
            .await?)
    }

    /// Creates or replaces the settings document.
    pub async fn save_settings(&self, settings: &Settings) -> Result<(), DbError> {
        let options = ReplaceOptions::builder().upsert(true).build();
        self.settings()
            .replace_one(doc! { "_id": SETTINGS_ID }, settings)
            .with_options(options)
            .await?;
        Ok(())
    }

    /// Updates just the activity cursor within the settings document.
    pub async fn update_activity_cursor(&self, cursor: &ActivityCursor) -> Result<(), DbError> {
        let cursor_bson = mongodb::bson::to_bson(cursor)
            .map_err(|e| DbError::Serialization(e.to_string()))?;

        let result = self
            .settings()
            .update_one(
                doc! { "_id": SETTINGS_ID },
                doc! { "$set": { "activity_cursor": cursor_bson } },
            )
            .await?;

        if result.matched_count == 0 {
            return Err(DbError::NotFound(
                "Settings document not found. Save settings first.".to_string(),
            ));
        }
        Ok(())
    }

    /// Updates just the player name within the settings document.
    pub async fn update_player_name(&self, player_name: &str) -> Result<(), DbError> {
        let result = self
            .settings()
            .update_one(
                doc! { "_id": SETTINGS_ID },
                doc! { "$set": { "player_name": player_name } },
            )
            .await?;

        if result.matched_count == 0 {
            return Err(DbError::NotFound(
                "Settings document not found. Save settings first.".to_string(),
            ));
        }
        Ok(())
    }
}
