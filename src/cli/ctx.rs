//! Shared context for CLI command handlers
//!
//! Provides database pool, configuration, and repositories
//! to all command handlers.

use crate::config::Config;
use crate::db::{DbPool, SqliteTaskRepository, SqliteQuestionRepository, SqliteCommentRepository, SqliteEventRepository};
use crate::db::{TaskRepository, QuestionRepository, CommentRepository, EventRepository};
use crate::error::Result;

/// Shared context for CLI command execution
pub struct Ctx {
    pub config: Config,
    pub pool: DbPool,
}

impl Ctx {
    /// Initialize context from config path (or default)
    pub async fn init(config_path: Option<&std::path::PathBuf>) -> Result<Self> {
        let config = match config_path {
            Some(path) => Config::load_from(path)?,
            None => Config::load()?,
        };

        let pool = DbPool::connect(&config).await?;

        Ok(Self { config, pool })
    }

    /// Get task repository
    pub fn task_repo(&self) -> Box<dyn TaskRepository> {
        Box::new(SqliteTaskRepository::new(&self.pool))
    }

    /// Get question repository
    pub fn question_repo(&self) -> Box<dyn QuestionRepository> {
        Box::new(SqliteQuestionRepository::new(&self.pool))
    }

    /// Get comment repository
    pub fn comment_repo(&self) -> Box<dyn CommentRepository> {
        Box::new(SqliteCommentRepository::new(&self.pool))
    }

    /// Get event repository
    pub fn event_repo(&self) -> Box<dyn EventRepository> {
        Box::new(SqliteEventRepository::new(&self.pool))
    }
}
