//! Events module - audit log tracking all entity changes
//!
//! Events module - audit log tracking all entity changes

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Question, Task, TaskStatus};

/// Type of entity that an event relates to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Task,
    Question,
    Comment,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Task => write!(f, "TASK"),
            Self::Question => write!(f, "QUESTION"),
            Self::Comment => write!(f, "COMMENT"),
        }
    }
}

impl std::str::FromStr for EntityType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TASK" => Ok(Self::Task),
            "QUESTION" => Ok(Self::Question),
            "COMMENT" => Ok(Self::Comment),
            _ => Err(format!("Invalid entity type: {}", s)),
        }
    }
}

/// Type of event that occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    Created,
    Updated,
    StatusChanged,
    Answered,
    Deleted,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "CREATED"),
            Self::Updated => write!(f, "UPDATED"),
            Self::StatusChanged => write!(f, "STATUS_CHANGED"),
            Self::Answered => write!(f, "ANSWERED"),
            Self::Deleted => write!(f, "DELETED"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().replace("-", "_").as_str() {
            "CREATED" => Ok(Self::Created),
            "UPDATED" => Ok(Self::Updated),
            "STATUS_CHANGED" | "STATUSCHANGED" => Ok(Self::StatusChanged),
            "ANSWERED" => Ok(Self::Answered),
            "DELETED" => Ok(Self::Deleted),
            _ => Err(format!("Invalid event type: {}", s)),
        }
    }
}

/// Core Event entity for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl Event {
    /// Create an event for task creation
    pub fn task_created(task: &Task) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: EntityType::Task,
            entity_id: task.id,
            event_type: EventType::Created,
            payload: serde_json::to_value(task).unwrap_or(serde_json::json!({})),
            created_at: Utc::now(),
        }
    }

    /// Create an event for task status change
    pub fn task_status_changed(task_id: Uuid, old: TaskStatus, new: TaskStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: EntityType::Task,
            entity_id: task_id,
            event_type: EventType::StatusChanged,
            payload: serde_json::json!({"old": old, "new": new}),
            created_at: Utc::now(),
        }
    }

    /// Create an event for question creation
    pub fn question_asked(question: &Question) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: EntityType::Question,
            entity_id: question.id,
            event_type: EventType::Created,
            payload: serde_json::to_value(question).unwrap_or(serde_json::json!({})),
            created_at: Utc::now(),
        }
    }

    /// Create an event for question answer
    pub fn question_answered(question_id: Uuid, answer: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type: EntityType::Question,
            entity_id: question_id,
            event_type: EventType::Answered,
            payload: serde_json::json!({"answer": answer}),
            created_at: Utc::now(),
        }
    }
}
