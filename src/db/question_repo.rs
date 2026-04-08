//! SQLite implementation of QuestionRepository
//!
//! Provides type-safe database operations for questions using sqlx.

use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::core::*;
use crate::db::{DbPool, QuestionRepository as QuestionRepositoryTrait};
use crate::error::{Error, Result};

/// SQLite-based question repository implementation
pub struct SqliteQuestionRepository {
    pool: SqlitePool,
}

impl SqliteQuestionRepository {
    /// Create a new SQLite question repository
    pub fn new(pool: &DbPool) -> Self {
        Self {
            pool: pool.as_sqlite().clone(),
        }
    }
}

#[async_trait]
impl QuestionRepositoryTrait for SqliteQuestionRepository {
    async fn create(&self, task_id: uuid::Uuid, input: &CreateQuestion) -> Result<Question> {
        let id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO questions (id, task_id, question_type, question_text, options, answer, answered_at, answered_by, urgency, created_at)
            VALUES (?, ?, ?, ?, ?, NULL, NULL, NULL, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(task_id.to_string())
        .bind(input.question_type.to_string())
        .bind(&input.question_text)
        .bind(input.options.as_ref().map(|o| serde_json::to_string(o).unwrap_or_default()))
        .bind(input.urgency.to_string())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(Question {
            id,
            task_id,
            question_type: input.question_type,
            question_text: input.question_text.clone(),
            options: input.options.clone(),
            answer: None,
            answered_at: None,
            answered_by: None,
            urgency: input.urgency,
            created_at: now,
        })
    }

    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Question>> {
        let row = sqlx::query_as::<_, (
            String, String, String, String, Option<String>, Option<String>,
            Option<String>, Option<String>, String, String
        )>(
            "SELECT id, task_id, question_type, question_text, options, answer, answered_at, answered_by, urgency, created_at FROM questions WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((id_str, task_id, question_type, question_text, options, answer, answered_at, answered_by, urgency, created_at)) => {
                Ok(Some(Question {
                    id: uuid::Uuid::parse_str(&id_str).map_err(|e| Error::Parse(e.to_string()))?,
                    task_id: uuid::Uuid::parse_str(&task_id).map_err(|e| Error::Parse(e.to_string()))?,
                    question_type: question_type.parse().map_err(|e: String| Error::Parse(e))?,
                    question_text,
                    options: options.and_then(|s| serde_json::from_str(&s).ok()),
                    answer,
                    answered_at: answered_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    answered_by,
                    urgency: urgency.parse().map_err(|e: String| Error::Parse(e))?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .map_err(|e| Error::Parse(e.to_string()))?
                        .with_timezone(&chrono::Utc),
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_task(&self, task_id: uuid::Uuid) -> Result<Vec<Question>> {
        let rows = sqlx::query_as::<_, (
            String, String, String, String, Option<String>, Option<String>,
            Option<String>, Option<String>, String, String
        )>(
            "SELECT id, task_id, question_type, question_text, options, answer, answered_at, answered_by, urgency, created_at FROM questions WHERE task_id = ? ORDER BY created_at DESC"
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|(id, task_id, question_type, question_text, options, answer, answered_at, answered_by, urgency, created_at)| {
            Ok(Question {
                id: uuid::Uuid::parse_str(&id).map_err(|e| Error::Parse(e.to_string()))?,
                task_id: uuid::Uuid::parse_str(&task_id).map_err(|e| Error::Parse(e.to_string()))?,
                question_type: question_type.parse().map_err(|e: String| Error::Parse(e))?,
                question_text,
                options: options.and_then(|s| serde_json::from_str(&s).ok()),
                answer,
                answered_at: answered_at.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                answered_by,
                urgency: urgency.parse().map_err(|e: String| Error::Parse(e))?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| Error::Parse(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            })
        }).collect()
    }

    async fn answer(&self, id: uuid::Uuid, input: &AnswerQuestion) -> Result<Question> {
        let mut question = self.find_by_id(id).await?
            .ok_or(Error::QuestionNotFound(id))?;

        let now = chrono::Utc::now();
        question.answer = Some(input.answer.clone());
        question.answered_at = Some(now);
        question.answered_by = input.answered_by.clone();

        sqlx::query(
            "UPDATE questions SET answer = ?, answered_at = ?, answered_by = ? WHERE id = ?"
        )
        .bind(&question.answer)
        .bind(question.answered_at.map(|dt| dt.to_rfc3339()))
        .bind(&question.answered_by)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(question)
    }
}
