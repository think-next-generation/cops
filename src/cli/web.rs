//! Web server command handler
//!
//! Starts HTTP/WebSocket server with optional SPA frontend.

use std::net::SocketAddr;

use super::args::WebCommands;
use super::ctx::Ctx;
use crate::api::{create_app_router, ApiState};
use crate::error::{Error, Result};
use crate::ws::Broadcaster;

pub async fn handle(cmd: WebCommands, ctx: &Ctx) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", cmd.host, cmd.port)
        .parse()
        .map_err(|e| Error::Config(format!("Invalid address: {}", e)))?;

    let state = ApiState::new(ctx.config.clone(), ctx.pool.clone());
    let broadcaster = Broadcaster::new();

    let app = create_app_router(state, broadcaster);

    println!("Starting web server...");
    println!("  Address: http://{}", addr);
    println!(
        "  WebSocket: {}",
        if ctx.config.server.websocket_enabled {
            "enabled"
        } else {
            "disabled"
        }
    );

    if cmd.no_ui {
        println!("  Mode: API only (no frontend)");
    } else {
        println!("  Frontend: embedded Vue 3 SPA");
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| Error::Io(e))?;

    println!();
    println!("Server running at http://{}", addr);

    if cmd.open {
        println!("Opening browser...");
        if let Err(e) = open::that(format!("http://{}", addr)) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    axum::serve(listener, app)
        .await
        .map_err(|e| Error::Custom(format!("Server error: {}", e)))?;

    Ok(())
}
