//! Comment API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::api::{ApiResponse, ApiState};
use crate::core::{AuthorType, Comment, CreateComment};
use crate::db::{CommentRepository, TaskRepository};

/// List comments for a specific task
///
/// GET /api/v1/tasks/:task_id/comments
pub async fn list_comments(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
) -> ApiResponse<Vec<Comment>> {
    let repo = crate::db::SqliteCommentRepository::new(&state.pool);

    match repo.find_by_task(task_id).await {
        Ok(comments) => ApiResponse::success(comments),
        Err(e) => ApiResponse::error(format!("Failed to list comments: {}", e)),
    }
}

/// Create a new comment on a task
///
/// POST /api/v1/tasks/:task_id/comments
pub async fn create_comment(
    State(state): State<ApiState>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<CreateComment>,
) -> ApiResponse<Comment> {
    // Validate content
    let content = input.content.trim();
    if content.is_empty() {
        return ApiResponse::error("Comment content cannot be empty");
    }
    if content.len() > 10000 {
        return ApiResponse::error("Comment content too long (max 10000 chars)");
    }

    // Verify task exists
    let task_repo = crate::db::SqliteTaskRepository::new(&state.pool);
    match task_repo.find_by_id(task_id).await {
        Ok(Some(_)) => {}, // Task exists, continue
        Ok(None) => return ApiResponse::error(format!("Task not found: {}", task_id)),
        Err(e) => return ApiResponse::error(format!("Failed to verify task: {}", e)),
    }

    // Create validated input with defaults
    let validated_input = CreateComment {
        content: content.to_string(),
        author_id: input.author_id.or_else(|| Some("system".to_string())),
        author_type: input.author_type.or(Some(AuthorType::Agent)),
    };

    let repo = crate::db::SqliteCommentRepository::new(&state.pool);

    match repo.create(task_id, &validated_input).await {
        Ok(comment) => ApiResponse::success(comment),
        Err(e) => ApiResponse::error(format!("Failed to create comment: {}", e)),
    }
}
