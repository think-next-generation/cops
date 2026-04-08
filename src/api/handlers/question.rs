//! Question API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::api::{ApiResponse, ApiState};
use crate::core::{AnswerQuestion, CreateQuestion, Question, QuestionType};
use crate::db::{QuestionRepository, TaskRepository};

/// List all questions with optional task filter
///
/// GET /api/v1/questions
pub async fn list_questions(_state: State<ApiState>) -> ApiResponse<Vec<Question>> {
    // Global question listing not yet implemented.
    // Use GET /api/v1/tasks/:task_id/questions to list questions for a specific task.
    ApiResponse::error("Global question listing not yet implemented. Use /api/v1/tasks/:task_id/questions to list questions for a specific task.")
}

/// List questions for a specific task
///
/// GET /api/v1/tasks/:task_id/questions
pub async fn list_task_questions(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> ApiResponse<Vec<Question>> {
    let repo = crate::db::SqliteQuestionRepository::new(&state.pool);

    match repo.find_by_task(task_id).await {
        Ok(questions) => ApiResponse::success(questions),
        Err(e) => ApiResponse::error(format!("Failed to list questions: {}", e)),
    }
}

/// Create a new question for a task
///
/// POST /api/v1/tasks/:task_id/questions
pub async fn create_question(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<CreateQuestion>,
) -> ApiResponse<Question> {
    // Validate question text
    let question_text = input.question_text.trim();
    if question_text.is_empty() {
        return ApiResponse::error("Question text cannot be empty");
    }
    if question_text.len() > 5000 {
        return ApiResponse::error("Question text too long (max 5000 chars)");
    }

    // Validate options for choice questions
    if matches!(input.question_type, QuestionType::SingleChoice | QuestionType::MultiChoice) {
        if input.options.is_none() || input.options.as_ref().map_or(true, |o| o.is_empty()) {
            return ApiResponse::error("Choice questions require options");
        }
    }

    // Verify task exists
    let task_repo = crate::db::SqliteTaskRepository::new(&state.pool);
    match task_repo.find_by_id(task_id).await {
        Ok(Some(_)) => {}, // Task exists, continue
        Ok(None) => return ApiResponse::error(format!("Task not found: {}", task_id)),
        Err(e) => return ApiResponse::error(format!("Failed to verify task: {}", e)),
    }

    // Create validated input
    let validated_input = CreateQuestion {
        question_text: question_text.to_string(),
        question_type: input.question_type,
        options: input.options,
        urgency: input.urgency,
    };

    let repo = crate::db::SqliteQuestionRepository::new(&state.pool);

    match repo.create(task_id, &validated_input).await {
        Ok(question) => ApiResponse::success(question),
        Err(e) => ApiResponse::error(format!("Failed to create question: {}", e)),
    }
}

/// Answer a question
///
/// POST /api/v1/questions/:id/answer
pub async fn answer_question(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(input): Json<AnswerQuestion>,
) -> ApiResponse<Question> {
    // Validate answer
    let answer = input.answer.trim();
    if answer.is_empty() {
        return ApiResponse::error("Answer cannot be empty");
    }

    let repo = crate::db::SqliteQuestionRepository::new(&state.pool);

    // Check if question exists
    match repo.find_by_id(id).await {
        Ok(Some(question)) => {
            // Check if already answered
            if question.is_answered() {
                return ApiResponse::error("Question has already been answered");
            }

            // Create a new input with trimmed answer
            let answer_input = AnswerQuestion {
                answer: answer.to_string(),
                answered_by: input.answered_by,
            };

            match repo.answer(id, &answer_input).await {
                Ok(updated_question) => ApiResponse::success(updated_question),
                Err(e) => ApiResponse::error(format!("Failed to answer question: {}", e)),
            }
        }
        Ok(None) => ApiResponse::error(format!("Question not found: {}", id)),
        Err(e) => ApiResponse::error(format!("Failed to find question: {}", e)),
    }
}
