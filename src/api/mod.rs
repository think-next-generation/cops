//! API module - REST API and WebSocket handlers

pub mod error;
pub mod handlers;
pub mod routes;
pub mod state;

pub use error::{ApiError, ApiResponse};
pub use routes::{create_api_router, create_app_router};
pub use state::ApiState;
