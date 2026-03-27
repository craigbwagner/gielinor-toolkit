use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;

use crate::client::DbClient;
use crate::error::DbError;
use crate::models::Drop;

const COLLECTION: &str = "drops";

impl DbClient {
    fn drops(&self) -> Collection<Drop> {
        self.database().collection(COLLECTION)
    }

    /// Gets all drops for a specific boss, newest first.
    pub async fn get_drops_for_boss(&self, boss_slug: &str) -> Result<Vec<Drop>, DbError> {
        use mongodb::options::FindOptions;
        use tokio_stream::StreamExt;

        let options = FindOptions::builder()
            .sort(doc! { "received_at": -1 })
            .build();

        let mut cursor = self
            .drops()
            .find(doc! { "boss_slug": boss_slug })
            .with_options(options)
            .await?;

        let mut drops = Vec::new();
        while let Some(drop) = cursor.next().await {
            drops.push(drop?);
        }
        Ok(drops)
    }

    /// Logs a significant drop. Also adds its GP value to the boss's wealth.
    pub async fn log_drop(
        &self,
        boss_slug: &str,
        item_name: &str,
        gp_value: u64,
        received_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<Drop, DbError> {
        let now = Utc::now();
        let drop = Drop {
            id: None, // MongoDB will generate an ObjectId
            boss_slug: boss_slug.to_string(),
            item_name: item_name.to_string(),
            gp_value,
            received_at: received_at.unwrap_or(now),
            logged_at: now,
        };

        self.drops().insert_one(&drop).await?;

        // Also add the drop's value to the boss's total wealth
        self.add_boss_wealth(boss_slug, gp_value).await?;

        Ok(drop)
    }

    /// Deletes a drop by its ObjectId. Does NOT reverse the wealth addition.
    pub async fn delete_drop(&self, id: &mongodb::bson::oid::ObjectId) -> Result<(), DbError> {
        let result = self.drops().delete_one(doc! { "_id": id }).await?;
        if result.deleted_count == 0 {
            return Err(DbError::NotFound(format!("Drop not found: {}", id)));
        }
        Ok(())
    }
}
