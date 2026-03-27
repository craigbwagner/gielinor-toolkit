use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Collection;

use crate::client::DbClient;
use crate::error::DbError;
use crate::models::Boss;

const COLLECTION: &str = "bosses";

impl DbClient {
    fn bosses(&self) -> Collection<Boss> {
        self.database().collection(COLLECTION)
    }

    /// Gets all tracked bosses.
    pub async fn get_all_bosses(&self) -> Result<Vec<Boss>, DbError> {
        use tokio_stream::StreamExt;

        let mut cursor = self.bosses().find(doc! {}).await?;
        let mut bosses = Vec::new();
        while let Some(boss) = cursor.next().await {
            bosses.push(boss?);
        }
        Ok(bosses)
    }

    /// Gets a single boss by slug.
    pub async fn get_boss(&self, slug: &str) -> Result<Boss, DbError> {
        self.bosses()
            .find_one(doc! { "_id": slug })
            .await?
            .ok_or_else(|| DbError::NotFound(format!("Boss not found: {}", slug)))
    }

    /// Creates a new tracked boss with initial kill count.
    pub async fn create_boss(&self, slug: &str, name: &str, initial_kills: u32) -> Result<Boss, DbError> {
        let boss = Boss {
            slug: slug.to_string(),
            name: name.to_string(),
            kills: initial_kills,
            initial_kills,
            total_wealth: 0,
            tracked_since: Utc::now(),
        };
        self.bosses().insert_one(&boss).await?;
        Ok(boss)
    }

    /// Increments a boss's kill count by the given amount.
    pub async fn increment_boss_kills(&self, slug: &str, count: u32) -> Result<(), DbError> {
        let result = self
            .bosses()
            .update_one(
                doc! { "_id": slug },
                doc! { "$inc": { "kills": count as i64 } },
            )
            .await?;

        if result.matched_count == 0 {
            return Err(DbError::NotFound(format!("Boss not found: {}", slug)));
        }
        Ok(())
    }

    /// Adds GP to a boss's total wealth (for misc loot or significant drops).
    pub async fn add_boss_wealth(&self, slug: &str, gp: u64) -> Result<(), DbError> {
        let result = self
            .bosses()
            .update_one(
                doc! { "_id": slug },
                doc! { "$inc": { "total_wealth": gp as i64 } },
            )
            .await?;

        if result.matched_count == 0 {
            return Err(DbError::NotFound(format!("Boss not found: {}", slug)));
        }
        Ok(())
    }

    /// Deletes a tracked boss.
    pub async fn delete_boss(&self, slug: &str) -> Result<(), DbError> {
        let result = self.bosses().delete_one(doc! { "_id": slug }).await?;
        if result.deleted_count == 0 {
            return Err(DbError::NotFound(format!("Boss not found: {}", slug)));
        }
        Ok(())
    }
}
