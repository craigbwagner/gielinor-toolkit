use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Collection;

use crate::client::DbClient;
use crate::error::DbError;
use crate::models::{Goal, GoalType};

const COLLECTION: &str = "goals";

impl DbClient {
    fn goals(&self) -> Collection<Goal> {
        self.database().collection(COLLECTION)
    }

    /// Gets all goals, optionally filtered by category.
    pub async fn get_goals(&self, category: Option<&str>) -> Result<Vec<Goal>, DbError> {
        use tokio_stream::StreamExt;

        let filter = match category {
            Some(cat) => doc! { "category": cat },
            None => doc! {},
        };

        let mut cursor = self.goals().find(filter).await?;
        let mut goals = Vec::new();
        while let Some(goal) = cursor.next().await {
            goals.push(goal?);
        }
        Ok(goals)
    }

    /// Gets a single goal by ID.
    pub async fn get_goal(&self, id: &ObjectId) -> Result<Goal, DbError> {
        self.goals()
            .find_one(doc! { "_id": id })
            .await?
            .ok_or_else(|| DbError::NotFound(format!("Goal not found: {}", id)))
    }

    /// Creates a new goal.
    pub async fn create_goal(
        &self,
        title: &str,
        goal_type: GoalType,
        category: Option<String>,
        boss_slug: Option<String>,
        skill_id: Option<u8>,
    ) -> Result<Goal, DbError> {
        let goal = Goal {
            id: None,
            title: title.to_string(),
            goal_type,
            category,
            boss_slug,
            skill_id,
            created_at: Utc::now(),
        };
        self.goals().insert_one(&goal).await?;
        Ok(goal)
    }

    /// Updates a goal's type (e.g. update numeric progress, toggle checkbox).
    pub async fn update_goal_type(
        &self,
        id: &ObjectId,
        goal_type: &GoalType,
    ) -> Result<(), DbError> {
        let goal_type_bson = mongodb::bson::to_bson(goal_type)
            .map_err(|e| DbError::Serialization(e.to_string()))?;

        let result = self
            .goals()
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "goal_type": goal_type_bson } },
            )
            .await?;

        if result.matched_count == 0 {
            return Err(DbError::NotFound(format!("Goal not found: {}", id)));
        }
        Ok(())
    }

    /// Updates the `current` value on a Numeric goal.
    /// Returns an error if the goal is not Numeric.
    pub async fn update_numeric_progress(
        &self,
        id: &ObjectId,
        current: u64,
    ) -> Result<(), DbError> {
        let result = self
            .goals()
            .update_one(
                doc! { "_id": id, "goal_type.type": "Numeric" },
                doc! { "$set": { "goal_type.current": current as i64 } },
            )
            .await?;

        if result.matched_count == 0 {
            return Err(DbError::NotFound(format!(
                "Numeric goal not found: {}",
                id
            )));
        }
        Ok(())
    }

    /// Deletes a goal.
    pub async fn delete_goal(&self, id: &ObjectId) -> Result<(), DbError> {
        let result = self.goals().delete_one(doc! { "_id": id }).await?;
        if result.deleted_count == 0 {
            return Err(DbError::NotFound(format!("Goal not found: {}", id)));
        }
        Ok(())
    }
}
