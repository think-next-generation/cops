//! SQLite implementation of EventRepository
//!
//! Provides type-safe database operations for events using sqlx.

use async_trait::async_trait;
use sqlx::SqlitePool;
use chrono::{DateTime, Utc};

use crate::core::*;
use crate::db::{DbPool, EventRepository as EventRepositoryTrait};
use crate::error::{Error, Result};

/// SQLite-based event repository implementation
pub struct SqliteEventRepository {
    pool: SqlitePool,
}

impl SqliteEventRepository {
    /// Create a new SQLite event repository
    pub fn new(pool: &DbPool) -> Self {
        Self {
            pool: pool.as_sqlite().clone(),
        }
    }
}

#[async_trait]
impl EventRepositoryTrait for SqliteEventRepository {
    async fn append(&self, event: &Event) -> Result<()> {
        sqlx::query(
            "INSERT INTO events (id, entity_type, entity_id, event_type, payload, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(event.id.to_string())
        .bind(event.entity_type.to_string())
        .bind(event.entity_id.to_string())
        .bind(event.event_type.to_string())
        .bind(serde_json::to_string(&event.payload)?)
        .bind(event.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_since(&self, since: DateTime<Utc>) -> Result<Vec<Event>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
            "SELECT id, entity_type, entity_id, event_type, payload, created_at FROM events WHERE created_at >= ? ORDER BY created_at DESC"
        )
        .bind(since.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|(id, entity_type, entity_id, event_type, payload, created_at)| {
            Ok(Event {
                id: uuid::Uuid::parse_str(&id).map_err(|e| Error::Parse(e.to_string()))?,
                entity_type: entity_type.parse().map_err(|e: String| Error::Parse(e))?,
                entity_id: uuid::Uuid::parse_str(&entity_id).map_err(|e| Error::Parse(e.to_string()))?,
                event_type: event_type.parse().map_err(|e: String| Error::Parse(e))?,
                payload: serde_json::from_str(&payload).unwrap_or(serde_json::json!({})),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| Error::Parse(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            })
        }).collect()
    }
}
