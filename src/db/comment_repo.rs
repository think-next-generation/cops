//! SQLite implementation of CommentRepository
//!
//! Provides type-safe database operations for comments using sqlx.

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::core::*;
use crate::db::{DbPool, CommentRepository as CommentRepositoryTrait};
use crate::error::{Error, Result};

/// SQLite-based comment repository implementation
pub struct SqliteCommentRepository {
    pool: SqlitePool,
}

impl SqliteCommentRepository {
    /// Create a new SQLite comment repository
    pub fn new(pool: &DbPool) -> Self {
        Self {
            pool: pool.as_sqlite().clone(),
        }
    }
}

#[async_trait]
impl CommentRepositoryTrait for SqliteCommentRepository {
    async fn create(&self, task_id: uuid::Uuid, input: &CreateComment) -> Result<Comment> {
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();

        let author_id = input.author_id.clone().unwrap_or_else(|| "system".to_string());
        let author_type = input.author_type.unwrap_or(AuthorType::Agent);

        sqlx::query(
            "INSERT INTO comments (id, task_id, author_id, author_type, content, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id.to_string())
        .bind(task_id.to_string())
        .bind(&author_id)
        .bind(author_type.to_string())
        .bind(&input.content)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(Comment {
            id,
            task_id,
            author_id,
            author_type,
            content: input.content.clone(),
            created_at: now,
        })
    }

    async fn find_by_task(&self, task_id: uuid::Uuid) -> Result<Vec<Comment>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
            "SELECT id, task_id, author_id, author_type, content, created_at FROM comments WHERE task_id = ? ORDER BY created_at ASC"
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|(id, task_id, author_id, author_type, content, created_at)| {
            Ok(Comment {
                id: uuid::Uuid::parse_str(&id).map_err(|e| Error::Parse(e.to_string()))?,
                task_id: uuid::Uuid::parse_str(&task_id).map_err(|e| Error::Parse(e.to_string()))?,
                author_id,
                author_type: author_type.parse().map_err(|e: String| Error::Parse(e))?,
                content,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| Error::Parse(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            })
        }).collect()
    }
}
