//! API routes configuration

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::Response,
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use tower_http::cors::CorsLayer;

use crate::api::handlers;
use crate::api::state::ApiState;
use crate::ws::Broadcaster;

#[derive(RustEmbed)]
#[folder = "src/frontend"]
struct FrontendAssets;

/// Serve frontend index.html
pub async fn serve_frontend() -> Response<Body> {
    match FrontendAssets::get("index.html") {
        Some(content) => {
            let mime = mime_guess::from_path("index.html").first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from("<h1>Frontend not found</h1>"))
            .unwrap(),
    }
}

/// Handle static assets
pub async fn handle_static(uri: Uri) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return serve_frontend().await;
    }

    match FrontendAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => serve_frontend().await,
    }
}

/// Create the full application router
pub fn create_app_router(state: ApiState, broadcaster: Broadcaster) -> Router {
    // API routes
    let api_router = Router::new()
        // Tasks
        .route(
            "/api/v1/tasks",
            get(handlers::task::list_tasks).post(handlers::task::create_task),
        )
        .route(
            "/api/v1/tasks/:id",
            get(handlers::task::get_task)
                .put(handlers::task::update_task)
                .delete(handlers::task::delete_task),
        )
        .route(
            "/api/v1/tasks/:id/move",
            get(handlers::task::move_task).post(handlers::task::move_task),
        )
        // Questions
        .route("/api/v1/questions", get(handlers::question::list_questions))
        .route(
            "/api/v1/questions/:id/answer",
            axum::routing::post(handlers::question::answer_question),
        )
        .route(
            "/api/v1/tasks/:task_id/questions",
            get(handlers::question::list_task_questions)
                .post(handlers::question::create_question),
        )
        // Comments
        .route(
            "/api/v1/tasks/:task_id/comments",
            get(handlers::comment::list_comments)
                .post(handlers::comment::create_comment),
        )
        // Status/Board
        .route("/api/v1/status", get(handlers::status::get_status))
        .route("/api/v1/board", get(handlers::status::get_board))
        // WebSocket
        .route(
            "/ws",
            get({
                let b = broadcaster.clone();
                move |ws| {
                    let b = b.clone();
                    async move { b.handle_ws(ws) }
                }
            }),
        )
        .with_state(state.clone());

    // Main router with fallback to frontend
    Router::new()
        .merge(api_router)
        .fallback(handle_static)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Create the API router with state (for backward compatibility)
pub fn create_api_router(state: ApiState) -> Router {
    create_app_router(state, Broadcaster::new())
}
