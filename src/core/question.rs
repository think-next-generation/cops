//! Question module - structured Q&A system with OPEN_ENDED, SINGLE_CHOICE, MULTI_CHOICE types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of question - determines how the question should be answered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuestionType {
    OpenEnded,
    SingleChoice,
    MultiChoice,
}

impl Default for QuestionType {
    fn default() -> Self {
        Self::OpenEnded
    }
}

/// Urgency level for questions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    Low,
    Normal,
    High,
}

impl Default for Urgency {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for QuestionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenEnded => write!(f, "OPEN_ENDED"),
            Self::SingleChoice => write!(f, "SINGLE_CHOICE"),
            Self::MultiChoice => write!(f, "MULTI_CHOICE"),
        }
    }
}

impl std::str::FromStr for QuestionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().replace("-", "_").as_str() {
            "OPEN_ENDED" | "OPENENDED" => Ok(Self::OpenEnded),
            "SINGLE_CHOICE" => Ok(Self::SingleChoice),
            "MULTI_CHOICE" => Ok(Self::MultiChoice),
            _ => Err(format!("Invalid question type: {}", s)),
        }
    }
}
impl std::fmt::Display for Urgency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
        }
    }
}
impl std::str::FromStr for Urgency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "normal" => Ok(Self::Normal),
            "high" => Ok(Self::High),
            _ => Err(format!("Invalid urgency: {}", s)),
        }
    }
}

/// Core Question entity for structured Q&A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: Uuid,
    pub task_id: Uuid,
    pub question_type: QuestionType,
    pub question_text: String,
    pub options: Option<Vec<String>>,
    pub answer: Option<String>,
    pub answered_at: Option<DateTime<Utc>>,
    pub answered_by: Option<String>,
    pub urgency: Urgency,
    pub created_at: DateTime<Utc>,
}

impl Question {
    /// Create a new question with the given task ID, text, and type
    pub fn new(task_id: Uuid, question_text: String, question_type: QuestionType) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            question_type,
            question_text,
            options: None,
            answer: None,
            answered_at: None,
            answered_by: None,
            urgency: Urgency::default(),
            created_at: Utc::now(),
        }
    }

    /// Check if this question has been answered
    pub fn is_answered(&self) -> bool {
        self.answer.is_some()
    }
}

/// Data required to create a new question
#[derive(Debug, Clone, Deserialize)]
pub struct CreateQuestion {
    pub question_text: String,
    #[serde(default)]
    pub question_type: QuestionType,
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub urgency: Urgency,
}

/// Data for answering a question
#[derive(Debug, Clone, Deserialize)]
pub struct AnswerQuestion {
    pub answer: String,
    pub answered_by: Option<String>,
}
