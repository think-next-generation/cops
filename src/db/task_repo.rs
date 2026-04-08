//! SQLite implementation of TaskRepository
//!
//! Provides type-safe database operations for tasks using sqlx.

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::core::*;
use crate::db::{DbPool, TaskRepository};
use crate::error::{Error, Result};

/// SQLite-based task repository implementation
pub struct SqliteTaskRepository {
    pool: SqlitePool,
}

impl SqliteTaskRepository {
    /// Create a new SQLite task repository
    pub fn new(pool: &DbPool) -> Self {
        Self {
            pool: pool.as_sqlite().clone(),
        }
    }
}

#[async_trait]
impl TaskRepository for SqliteTaskRepository {
    async fn create(&self, input: &CreateTask) -> Result<Task> {
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO tasks (id, title, description, status, priority, tags, assignees, blocked_by, parent_id, created_at, updated_at)
            VALUES (?, ?, ?, 'NEW', ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.priority.to_string())
        .bind(serde_json::to_string(&input.tags.clone().unwrap_or_default())?)
        .bind(serde_json::to_string(&input.assignees.clone().unwrap_or_default())?)
        .bind(serde_json::to_string(&input.blocked_by.clone().unwrap_or_default())?)
        .bind(input.parent_id.map(|id| id.to_string()))
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(Task {
            id,
            title: input.title.clone(),
            description: input.description.clone(),
            status: TaskStatus::New,
            priority: input.priority,
            tags: input.tags.clone().unwrap_or_default(),
            assignees: input.assignees.clone().unwrap_or_default(),
            blocked_by: input.blocked_by.clone().unwrap_or_default(),
            parent_id: input.parent_id,
            created_at: now,
            updated_at: now,
        })
    }

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Task>> {
        let row = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            String,
            String,
        )>(
            "SELECT id, title, description, status, priority, tags, assignees, blocked_by, parent_id, created_at, updated_at FROM tasks WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((id_str, title, description, status, priority, tags, assignees, blocked_by, parent_id, created_at, updated_at)) => {
                Ok(Some(Task {
                    id: uuid::Uuid::parse_str(&id_str).map_err(|e| Error::Parse(e.to_string()))?,
                    title,
                    description,
                    status: status.parse().map_err(|e: String| Error::Parse(e))?,
                    priority: priority.parse().unwrap_or(Priority::Medium),
                    tags: serde_json::from_str(&tags).unwrap_or_default(),
                    assignees: serde_json::from_str(&assignees).unwrap_or_default(),
                    blocked_by: serde_json::from_str(&blocked_by).unwrap_or_default(),
                    parent_id: parent_id.and_then(|s| uuid::Uuid::parse_str(&s).ok()),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map_err(|e| Error::Parse(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                        .map_err(|e| Error::Parse(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self, filters: &TaskFilters) -> Result<Vec<Task>> {
        let mut query = String::from(
            "SELECT id, title, description, status, priority, tags, assignees, blocked_by, parent_id, created_at, updated_at FROM tasks WHERE 1=1"
        );
        let mut params: Vec<String> = Vec::new();

        if let Some(ref statuses) = filters.status {
            if !statuses.is_empty() {
                let placeholders: Vec<&str> = statuses.iter().map(|_| "?").collect();
                query.push_str(&format!(" AND status IN ({})", placeholders.join(",")));
                for s in statuses {
                    params.push(s.to_string());
                }
            }
        }

        if let Some(ref assignee) = filters.assignee {
            query.push_str(" AND assignees LIKE ?");
            params.push(format!("%{}%", assignee));
        }

        if let Some(ref tag) = filters.tag {
            query.push_str(" AND tags LIKE ?");
            params.push(format!("%{}%", tag));
        }

        if let Some(parent) = filters.parent {
            query.push_str(" AND parent_id = ?");
            params.push(parent.to_string());
        }

        if let Some(blocked) = filters.blocked {
            if blocked {
                query.push_str(" AND blocked_by != '[]'");
            } else {
                query.push_str(" AND blocked_by = '[]'");
            }
        }

        query.push_str(" ORDER BY created_at DESC");

        // Build dynamic query with parameters
        let mut sql_query = sqlx::query_as::<_, (
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            String,
            String,
        )>(&query);

        for param in &params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        rows.into_iter()
            .map(|(id, title, description, status, priority, tags, assignees, blocked_by, parent_id, created_at, updated_at)| {
                Ok(Task {
                    id: uuid::Uuid::parse_str(&id).map_err(|e| Error::Parse(e.to_string()))?,
                    title,
                    description,
                    status: status.parse().map_err(|e: String| Error::Parse(e))?,
                    priority: priority.parse().unwrap_or(Priority::Medium),
                    tags: serde_json::from_str(&tags).unwrap_or_default(),
                    assignees: serde_json::from_str(&assignees).unwrap_or_default(),
                    blocked_by: serde_json::from_str(&blocked_by).unwrap_or_default(),
                    parent_id: parent_id.and_then(|s| uuid::Uuid::parse_str(&s).ok()),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map_err(|e| Error::Parse(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                        .map_err(|e| Error::Parse(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                })
            })
            .collect()
    }

    async fn update(&self, id: uuid::Uuid, input: &UpdateTask) -> Result<Task> {
        let mut task = self
            .find_by_id(id)
            .await?
            .ok_or(Error::TaskNotFound(id))?;
        let now = chrono::Utc::now();

        if let Some(ref title) = input.title {
            task.title = title.clone();
        }
        if let Some(ref desc) = input.description {
            task.description = Some(desc.clone());
        }
        if let Some(status) = input.status {
            if !task.can_transition_to(status) {
                return Err(Error::InvalidStatusTransition {
                    from: task.status.to_string(),
                    to: status.to_string(),
                });
            }
            task.status = status;
        }
        if let Some(priority) = input.priority {
            task.priority = priority;
        }
        if let Some(ref tags) = input.tags {
            task.tags = tags.clone();
        }
        task.updated_at = now;

        sqlx::query(
            r#"UPDATE tasks SET title = ?, description = ?, status = ?, priority = ?, tags = ?, assignees = ?, blocked_by = ?, updated_at = ? WHERE id = ?"#,
        )
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status.to_string())
        .bind(task.priority.to_string())
        .bind(serde_json::to_string(&task.tags)?)
        .bind(serde_json::to_string(&task.assignees)?)
        .bind(serde_json::to_string(&task.blocked_by)?)
        .bind(task.updated_at.to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(task)
    }

    async fn delete(&self, id: uuid::Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            Err(Error::TaskNotFound(id))
        } else {
            Ok(())
        }
    }

    async fn count_by_status(&self) -> Result<Vec<(TaskStatus, i64)>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT status, COUNT(*) as count FROM tasks GROUP BY status",
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(status, count)| Ok((status.parse().map_err(|e: String| Error::Parse(e))?, count)))
            .collect()
    }
}
