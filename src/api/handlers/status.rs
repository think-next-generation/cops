//! Status API handlers
//!
//! Provides system status and kanban board view endpoints.

use axum::extract::State;
use serde::Serialize;
use std::collections::HashMap;

use crate::api::{ApiResponse, ApiState};
use crate::core::TaskFilters;
use crate::db::TaskRepository;

/// Maximum number of tasks to show per column in board view
const MAX_TASKS_PER_COLUMN: usize = 50;

/// Response for system status endpoint
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Count of tasks grouped by status
    pub status_counts: HashMap<String, i64>,
    /// Total number of tasks
    pub total_tasks: i64,
    /// Database backend type
    pub database: String,
}

/// A single task in the board view (minimal data)
#[derive(Debug, Serialize)]
pub struct BoardTask {
    /// Task ID
    pub id: String,
    /// Task title
    pub title: String,
    /// Task priority
    pub priority: String,
}

/// A column in the kanban board
#[derive(Debug, Serialize)]
pub struct BoardColumn {
    /// Status name for this column
    pub status: String,
    /// Total count of tasks in this status
    pub count: i64,
    /// Tasks in this column (limited)
    pub tasks: Vec<BoardTask>,
}

/// Response for board view endpoint
#[derive(Debug, Serialize)]
pub struct BoardResponse {
    /// Columns in configured order
    pub columns: Vec<BoardColumn>,
    /// Total tasks across all columns
    pub total: i64,
}

/// Get system status with task counts by status
///
/// GET /api/v1/status
pub async fn get_status(State(state): State<ApiState>) -> ApiResponse<StatusResponse> {
    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    // Get status counts
    let status_counts_vec = match repo.count_by_status().await {
        Ok(counts) => counts,
        Err(e) => return ApiResponse::error(format!("Failed to get status counts: {}", e)),
    };

    // Convert to HashMap
    let mut status_counts: HashMap<String, i64> = HashMap::new();
    let mut total_tasks: i64 = 0;

    for (status, count) in status_counts_vec {
        status_counts.insert(status.to_string(), count);
        total_tasks += count;
    }

    // Get database backend
    let database = state.config.database.backend.clone();

    ApiResponse::success(StatusResponse {
        status_counts,
        total_tasks,
        database,
    })
}

/// Get kanban board view
///
/// GET /api/v1/board
pub async fn get_board(State(state): State<ApiState>) -> ApiResponse<BoardResponse> {
    let repo = crate::db::SqliteTaskRepository::new(&state.pool);

    // Get all tasks with default filters
    let tasks = match repo.find_all(&TaskFilters::default()).await {
        Ok(tasks) => tasks,
        Err(e) => return ApiResponse::error(format!("Failed to get tasks: {}", e)),
    };

    // Compute status counts and group tasks by status in a single pass
    let mut status_count_map: HashMap<String, i64> = HashMap::new();
    let mut tasks_by_status: HashMap<String, Vec<BoardTask>> = HashMap::new();

    for task in tasks {
        let status_key = task.status.to_string();

        // Increment count for this status
        *status_count_map.entry(status_key.clone()).or_insert(0) += 1;

        // Add task to grouping
        let board_task = BoardTask {
            id: task.id.to_string(),
            title: task.title,
            priority: task.priority.to_string(),
        };
        tasks_by_status
            .entry(status_key)
            .or_insert_with(Vec::new)
            .push(board_task);
    }

    // Build columns in configured order
    let columns: Vec<BoardColumn> = state
        .config
        .board
        .default_columns
        .iter()
        .map(|status| {
            let count = status_count_map.get(status).copied().unwrap_or(0);
            let mut tasks = tasks_by_status.remove(status).unwrap_or_default();
            // Limit tasks per column
            tasks.truncate(MAX_TASKS_PER_COLUMN);
            BoardColumn {
                status: status.clone(),
                count,
                tasks,
            }
        })
        .collect();

    // Calculate total
    let total: i64 = columns.iter().map(|c| c.count).sum();

    ApiResponse::success(BoardResponse { columns, total })
}
