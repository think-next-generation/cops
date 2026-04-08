//! Database module - SQLite/MariaDB persistence layer

mod pool;
mod repository;
mod task_repo;
mod question_repo;
mod comment_repo;
mod event_repo;

pub use pool::*;
pub use repository::{TaskRepository, QuestionRepository, CommentRepository, EventRepository};
pub use task_repo::SqliteTaskRepository;
pub use question_repo::SqliteQuestionRepository;
pub use comment_repo::SqliteCommentRepository;
pub use event_repo::SqliteEventRepository;
