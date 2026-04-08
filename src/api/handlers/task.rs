//! Task API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::{ApiResponse, ApiState};
use crate::core::{CreateTask, Task, TaskFilters, TaskStatus, UpdateTask};
use crate::db::TaskRepository;

/// Request body for moving a task to a new status
#[derive(Debug, Deserialize)]
pub struct MoveTaskRequest {
    pub status: TaskStatus,
}

/// Response for move_task operation
#[derive(Debug, Serialize)]
pub struct MoveTaskResponse {
    pub id: Uuid,
    pub status: TaskStatus,
    pub message: String,
}

/// List all tasks with optional filtering
///
/// GET /api/v1/tasks
pub async fn list_tasks(
    State(state): State<ApiState>,
    Query(filters): Query<TaskFilters>,
) -> ApiResponse<Vec<Task>> {
    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    match repo.find_all(&filters).await {
        Ok(tasks) => ApiResponse::success(tasks),
        Err(e) => ApiResponse::error(format!("Failed to list tasks: {}", e)),
    }
}

/// Create a new task
///
/// POST /api/v1/tasks
pub async fn create_task(
    State(state): State<ApiState>,
    Json(input): Json<CreateTask>,
) -> ApiResponse<Task> {
    // Validate title
    let title = input.title.trim();
    if title.is_empty() {
        return ApiResponse::error("Task title cannot be empty");
    }
    if title.len() > 500 {
        return ApiResponse::error("Task title too long (max 500 chars)");
    }

    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    match repo.create(&input).await {
        Ok(task) => ApiResponse::success(task),
        Err(e) => ApiResponse::error(format!("Failed to create task: {}", e)),
    }
}

/// Get a single task by ID
///
/// GET /api/v1/tasks/:id
pub async fn get_task(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Task> {
    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    match repo.find_by_id(id).await {
        Ok(Some(task)) => ApiResponse::success(task),
        Ok(None) => ApiResponse::error(format!("Task not found: {}", id)),
        Err(e) => ApiResponse::error(format!("Failed to get task: {}", e)),
    }
}

/// Update an existing task
///
/// PUT /api/v1/tasks/:id
pub async fn update_task(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTask>,
) -> ApiResponse<Task> {
    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    match repo.update(id, &input).await {
        Ok(task) => ApiResponse::success(task),
        Err(e) => ApiResponse::error(format!("Failed to update task: {}", e)),
    }
}

/// Move a task to a new status (with transition validation)
///
/// POST /api/v1/tasks/:id/move
pub async fn move_task(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(input): Json<MoveTaskRequest>,
) -> ApiResponse<MoveTaskResponse> {
    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    // Fetch current task first to validate transition
    let current = match repo.find_by_id(id).await {
        Ok(Some(task)) => task,
        Ok(None) => return ApiResponse::error(format!("Task not found: {}", id)),
        Err(e) => return ApiResponse::error(format!("Failed to fetch task: {}", e)),
    };

    // Validate status transition
    if !current.can_transition_to(input.status) {
        return ApiResponse::error(format!(
            "Invalid status transition: {} -> {}",
            current.status, input.status
        ));
    }

    // Build UpdateTask with only the status change
    let update = UpdateTask {
        title: None,
        description: None,
        status: Some(input.status),
        priority: None,
        tags: None,
    };

    match repo.update(id, &update).await {
        Ok(task) => ApiResponse::success(MoveTaskResponse {
            id: task.id,
            status: task.status,
            message: format!("Task moved to {}", task.status),
        }),
        Err(e) => ApiResponse::error(format!("Failed to move task: {}", e)),
    }
}

/// Delete a task
///
/// DELETE /api/v1/tasks/:id
pub async fn delete_task(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    match repo.delete(id).await {
        Ok(()) => ApiResponse::success(()),
        Err(e) => ApiResponse::error(format!("Failed to delete task: {}", e)),
    }
}
