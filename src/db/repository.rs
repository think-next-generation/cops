//! Repository traits for task management
//!
//! Defines the interface for persistence operations following the repository pattern.
//! Implementations can swap between SQLite, MySQL, or other storage backends.

use async_trait::async_trait;
use crate::core::*;
use crate::error::Result;

/// Repository trait for Task CRUD operations
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// Create a new task
    async fn create(&self, task: &CreateTask) -> Result<Task>;

    /// Find a task by its ID
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Task>>;

    /// Find all tasks matching the given filters
    async fn find_all(&self, filters: &TaskFilters) -> Result<Vec<Task>>;

    /// Update an existing task
    async fn update(&self, id: uuid::Uuid, update: &UpdateTask) -> Result<Task>;

    /// Delete a task by its ID
    async fn delete(&self, id: uuid::Uuid) -> Result<()>;

    /// Count tasks grouped by status
    async fn count_by_status(&self) -> Result<Vec<(TaskStatus, i64)>>;
}

/// Repository trait for Question CRUD operations
#[async_trait]
pub trait QuestionRepository: Send + Sync {
    /// Create a new question for a task
    async fn create(&self, task_id: uuid::Uuid, question: &CreateQuestion) -> Result<Question>;

    /// Find a question by its ID
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<Question>>;

    /// Find all questions for a given task
    async fn find_by_task(&self, task_id: uuid::Uuid) -> Result<Vec<Question>>;

    /// Answer a question
    async fn answer(&self, id: uuid::Uuid, answer: &AnswerQuestion) -> Result<Question>;
}

/// Repository trait for Comment CRUD operations
#[async_trait]
pub trait CommentRepository: Send + Sync {
    /// Create a new comment for a task
    async fn create(&self, task_id: uuid::Uuid, comment: &CreateComment) -> Result<Comment>;

    /// Find all comments for a given task
    async fn find_by_task(&self, task_id: uuid::Uuid) -> Result<Vec<Comment>>;
}

/// Repository trait for Event operations
#[async_trait]
pub trait EventRepository: Send + Sync {
    /// Append an event to the audit log
    async fn append(&self, event: &Event) -> Result<()>;

    /// Find all events since a given timestamp
    async fn find_since(&self, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<Event>>;
}
