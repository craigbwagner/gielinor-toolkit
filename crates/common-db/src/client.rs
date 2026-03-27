use mongodb::{Client, Database, bson::doc};

use crate::error::DbError;

/// Database client wrapping a MongoDB connection.
/// Created once at app startup and shared across all repos.
pub struct DbClient {
    db: Database,
}

impl DbClient {
    /// Connects to MongoDB and verifies the connection with a ping.
    pub async fn new(connection_string: &str, db_name: &str) -> Result<Self, DbError> {
        let client = Client::with_uri_str(connection_string).await?;
        let db = client.database(db_name);

        // Verify the connection is alive
        db.run_command(doc! { "ping": 1 }).await?;

        Ok(Self { db })
    }

    /// Returns a handle to the underlying MongoDB database.
    /// Used by repo modules to get typed collection handles.
    pub(crate) fn database(&self) -> &Database {
        &self.db
    }
}
