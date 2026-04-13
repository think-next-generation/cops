//! Agent command handlers

use super::args::AgentCommands;
use super::ctx::Ctx;
use crate::core::{TaskStatus, TaskFilters};
use crate::error::Result;

pub async fn handle(cmd: AgentCommands, ctx: &Ctx) -> Result<()> {
    match cmd {
        AgentCommands::Status { agent } => handle_status(ctx, agent).await,
    }
}

async fn handle_status(ctx: &Ctx, agent_filter: Option<String>) -> Result<()> {
    let repo = ctx.task_repo();

    if let Some(agent_id) = agent_filter {
        // Show specific agent status
        let filters = TaskFilters {
            status: Some(vec![TaskStatus::Assigned, TaskStatus::InProgress, TaskStatus::Blocked, TaskStatus::Waiting]),
            assignee: Some(agent_id.clone()),
            ..Default::default()
        };
        let tasks = repo.find_all(&filters).await?;

        println!("Agent: {}", agent_id);
        println!("Active tasks: {}", tasks.len());
        println!();

        if tasks.is_empty() {
            println!("This agent has no active tasks.");
        } else {
            println!("{:<10} {:<40} {}", "Status", "Title", "ID");
            println!("{}", "-".repeat(70));
            for task in &tasks {
                let title = if task.title.len() > 38 {
                    format!("{}...", &task.title[..35])
                } else {
                    task.title.clone()
                };
                let id_short = &task.id.to_string()[..8.min(task.id.to_string().len())];
                println!("{:<10} {:<40} {}", task.status, title, id_short);
            }
        }
    } else {
        // Show all agents status
        let counts = repo.count_by_assignee().await?;

        if counts.is_empty() {
            println!("No active tasks with assignees.");
            return Ok(());
        }

        // Group by assignee
        use std::collections::HashMap;
        let mut agent_stats: HashMap<String, Vec<(TaskStatus, i64)>> = HashMap::new();
        for (agent_id, status, count) in counts {
            agent_stats.entry(agent_id).or_default().push((status, count));
        }

        println!("Agent Workload Summary");
        println!("======================");
        println!();
        println!("{:<20} {:<15} {:<15} {:<15} {:<15}", "Agent", "ASSIGNED", "IN_PROGRESS", "BLOCKED", "WAITING");
        println!("{}", "-".repeat(80));

        for (agent_id, statuses) in agent_stats.iter() {
            let mut assigned = 0i64;
            let mut in_progress = 0i64;
            let mut blocked = 0i64;
            let mut waiting = 0i64;

            for (status, count) in statuses {
                match status {
                    TaskStatus::Assigned => assigned = *count,
                    TaskStatus::InProgress => in_progress = *count,
                    TaskStatus::Blocked => blocked = *count,
                    TaskStatus::Waiting => waiting = *count,
                    _ => {}
                }
            }

            let busy = in_progress > 0;
            let busy_marker = if busy { " [BUSY]" } else { "" };

            println!("{:<20} {:<15} {:<15} {:<15} {:<15}{}",
                agent_id, assigned, in_progress, blocked, waiting, busy_marker);
        }

        // Summary - recalculate from agent_stats
        let mut total_assigned = 0i64;
        let mut total_in_progress = 0i64;
        let mut total_blocked = 0i64;
        let mut total_waiting = 0i64;

        for statuses in agent_stats.values() {
            for (status, count) in statuses {
                match status {
                    TaskStatus::Assigned => total_assigned += *count,
                    TaskStatus::InProgress => total_in_progress += *count,
                    TaskStatus::Blocked => total_blocked += *count,
                    TaskStatus::Waiting => total_waiting += *count,
                    _ => {}
                }
            }
        }

        println!("{:<20} {:<15} {:<15} {:<15} {:<15}",
            "TOTAL", total_assigned, total_in_progress, total_blocked, total_waiting);
    }

    Ok(())
}
