//! Status command handlers
//!
//! System status and monitoring.

use super::args::StatusCommands;
use super::ctx::Ctx;
use crate::error::{Error, Result};

pub async fn handle(cmd: StatusCommands, ctx: &Ctx) -> Result<()> {
    match cmd {
        StatusCommands::Show {
            format,
            since,
        } => handle_show(ctx, format, since).await,
        StatusCommands::Watch => handle_watch(ctx).await,
    }
}

async fn handle_show(
    ctx: &Ctx,
    format: String,
    since: Option<String>,
) -> Result<()> {
    let repo = ctx.task_repo();
    let status_counts = repo.count_by_status().await?;

    // Get recent events if since is specified
    let recent_events = if let Some(ref since_str) = since {
        let since_time = chrono::DateTime::parse_from_rfc3339(since_str)
            .map_err(|e| Error::Parse(format!("Invalid timestamp: {}", e)))?
            .with_timezone(&chrono::Utc);

        let event_repo = ctx.event_repo();
        Some(event_repo.find_since(since_time).await?)
    } else {
        None
    };

    match format.as_str() {
        "json" => {
            let status_map: std::collections::HashMap<String, i64> = status_counts
                .iter()
                .map(|(s, c)| (s.to_string(), *c))
                .collect();

            let output = serde_json::json!({
                "status_counts": status_map,
                "total_tasks": status_counts.iter().map(|(_, c)| c).sum::<i64>(),
                "recent_events": recent_events.map(|events| events.len()),
            });

            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            println!("COPS Status");
            println!("===========");
            println!();
            println!("Task Counts by Status:");
            for (status, count) in &status_counts {
                println!("  {}: {}", status, count);
            }
            let total: i64 = status_counts.iter().map(|(_, c)| c).sum();
            println!("  ─────────────");
            println!("  Total: {}", total);

            if let Some(events) = recent_events {
                println!();
                println!("Recent Events: {}", events.len());
            }
        }
    }

    Ok(())
}

async fn handle_watch(_ctx: &Ctx) -> Result<()> {
    println!("Watch mode streams events in real-time.");
    println!("This feature requires a running web server with WebSocket support.");
    println!();
    println!("Start the web server with: cops web");
    println!("Then connect to ws://localhost:9090/ws for real-time updates.");

    Ok(())
}
