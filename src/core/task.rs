use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task status - 8 state lifecycle
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    #[default]
    New,
    Assigned,
    InProgress,
    Blocked,
    Waiting,
    Review,
    Done,
    Archived,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "NEW"),
            Self::Assigned => write!(f, "ASSIGNED"),
            Self::InProgress => write!(f, "IN_PROGRESS"),
            Self::Blocked => write!(f, "BLOCKED"),
            Self::Waiting => write!(f, "WAITING"),
            Self::Review => write!(f, "REVIEW"),
            Self::Done => write!(f, "DONE"),
            Self::Archived => write!(f, "ARCHIVED"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().replace("-", "_").as_str() {
            "NEW" => Ok(Self::New),
            "ASSIGNED" => Ok(Self::Assigned),
            "IN_PROGRESS" | "INPROGRESS" => Ok(Self::InProgress),
            "BLOCKED" => Ok(Self::Blocked),
            "WAITING" => Ok(Self::Waiting),
            "REVIEW" => Ok(Self::Review),
            "DONE" => Ok(Self::Done),
            "ARCHIVED" => Ok(Self::Archived),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

/// Task priority levels
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Urgent,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Urgent => write!(f, "urgent"),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "urgent" => Ok(Self::Urgent),
            _ => Err(format!("Invalid priority: {}", s)),
        }
    }
}

/// Type of assignee (agent or human)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AssigneeType {
    Agent,
    Human,
}

/// Role of an assignee on a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssigneeRole {
    Primary,
    Reviewer,
    Approver,
    Contributor,
}

/// Task assignee with role information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignee {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: AssigneeType,
    pub role: AssigneeRole,
}

/// Core Task entity with 8-state lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub assignees: Vec<Assignee>,
    pub blocked_by: Vec<Uuid>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// Create a new task with the given title
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            status: TaskStatus::default(),
            priority: Priority::default(),
            tags: Vec::new(),
            assignees: Vec::new(),
            blocked_by: Vec::new(),
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if a transition to the new status is valid
    pub fn can_transition_to(&self, new_status: TaskStatus) -> bool {
        use TaskStatus::*;
        match self.status {
            New => matches!(new_status, Assigned),
            Assigned => matches!(new_status, InProgress | Blocked | Waiting | Review),
            InProgress => matches!(new_status, Blocked | Waiting | Review | Done),
            Blocked => matches!(new_status, Waiting | InProgress | Assigned),
            Waiting => matches!(new_status, Review | InProgress | Assigned),
            Review => matches!(new_status, Done | Waiting | InProgress),
            Done => matches!(new_status, Archived),
            Archived => false,
        }
    }
}

/// Data required to create a new task
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTask {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    pub tags: Option<Vec<String>>,
    pub assignees: Option<Vec<Assignee>>,
    pub blocked_by: Option<Vec<Uuid>>,
    pub parent_id: Option<Uuid>,
}

/// Data for updating an existing task
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub tags: Option<Vec<String>>,
}

/// Filters for querying tasks
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TaskFilters {
    pub status: Option<Vec<TaskStatus>>,
    pub assignee: Option<String>,
    pub tag: Option<String>,
    pub parent: Option<Uuid>,
    pub blocked: Option<bool>,
}
