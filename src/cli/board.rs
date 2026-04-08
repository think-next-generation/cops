//! Board command handlers

//!
//! Kanban-style board visualization.

use super::args::BoardCommands;
use super::ctx::Ctx;
use crate::core::{TaskFilters, TaskStatus};
use crate::error::{Error, Result};

use crate::error::Result as CliResult;

pub async fn handle(cmd: BoardCommands, ctx: &Ctx) -> CliResult<()> {
    match cmd {
        BoardCommands::Show {
            filter,
            assignee,
            watch,
        } => handle_show(ctx, filter, assignee, watch).await,
    }
}

async fn handle_show(
    ctx: &Ctx,
    filter: Option<String>,
    assignee: Option<String>,
    watch: bool,
) -> Result<()> {
    if watch {
        println!("Watch mode not yet implemented. Showing board once.");
    }

    let repo = ctx.task_repo();

    // Get status filter if provided
    let status_filter: Option<Vec<TaskStatus>> = filter
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: String| Error::Parse(e))?
        .map(|s| vec![s]);

    let filters = TaskFilters {
        status: status_filter,
        assignee: assignee.clone(),
        ..Default::default()
    };

    let tasks = repo.find_all(&filters).await?;
    let status_counts = repo.count_by_status().await?;

    // Build kanban view
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                          COPS TASK BOARD                                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // Show status columns
    let columns = &ctx.config.board.default_columns;

    for column in columns {
        let count = status_counts.iter()
            .find(|(status, _)| status.to_string() == *column)
            .map(|(_, c)| *c)
            .unwrap_or(0);

        let dash_len = 60usize.saturating_sub(column.len()).saturating_sub(count.to_string().len());
        println!("┌─ {} ({}) {}", column, count, "─".repeat(dash_len));

        let column_tasks: Vec<_> = tasks.iter()
            .filter(|t| t.status.to_string() == *column)
            .collect();

        if column_tasks.is_empty() {
            println!("│ (empty)");
        } else {
            for task in column_tasks.iter().take(5) {
                let id_str = task.id.to_string();
                let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
                let title = if task.title.len() > 50 {
                    format!("{}...", &task.title[..47.min(task.title.len())])
                } else {
                    task.title.clone()
                };
                println!("│ [{}] {}", id_short, title);
            }
            if column_tasks.len() > 5 {
                println!("│ ... and {} more", column_tasks.len() - 5);
            }
        }
        println!("└{}", "─".repeat(70));
        println!();
    }

    // Show summary
    println!("Summary:");
    for (status, count) in &status_counts {
        println!("  {}: {}", status, count);
    }
    println!("  Total: {}", tasks.len());

    Ok(())
}
