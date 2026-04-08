//! Comment module - free-form thread with AGENT/HUMAN author types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of author - distinguishes between agent and human comments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "UPPERCASE")]
pub enum AuthorType {
    Agent,
    Human,
}

impl Default for AuthorType {
    fn default() -> Self {
        Self::Agent
    }
}

impl std::fmt::Display for AuthorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agent => write!(f, "AGENT"),
            Self::Human => write!(f, "HUMAN"),
        }
    }
}
impl std::str::FromStr for AuthorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "AGENT" => Ok(Self::Agent),
            "HUMAN" => Ok(Self::Human),
            _ => Err(format!("Invalid author type: {}", s)),
        }
    }
}

/// Core Comment entity for task discussions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub author_id: String,
    pub author_type: AuthorType,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Comment {
    /// Create a new comment
    pub fn new(
        task_id: Uuid,
        author_id: String,
        author_type: AuthorType,
        content: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            author_id,
            author_type,
            content,
            created_at: Utc::now(),
        }
    }
}

/// Data required to create a new comment
#[derive(Debug, Clone, Deserialize)]
pub struct CreateComment {
    pub content: String,
    pub author_id: Option<String>,
    pub author_type: Option<AuthorType>,
}
