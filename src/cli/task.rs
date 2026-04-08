//! Task command handlers

use super::args::TaskCommands;
use super::ctx::Ctx;
use super::output::{self, Format};
use crate::core::{CreateTask, UpdateTask, TaskFilters, TaskStatus, Priority, Assignee, AssigneeType, AssigneeRole};
use crate::error::{Error, Result};
use uuid::Uuid;
use std::io::Write;

pub async fn handle(cmd: TaskCommands, ctx: &Ctx) -> Result<()> {
    match cmd {
        TaskCommands::Create {
            title,
            description,
            assignee,
            parent,
            blocked_by,
            priority,
            tag,
        } => handle_create(ctx, title, description, assignee, parent, blocked_by, priority, tag).await,
        TaskCommands::List {
            status,
            assignee,
            tag,
            parent,
            blocked,
            format,
        } => handle_list(ctx, status, assignee, tag, parent, blocked, format).await,
        TaskCommands::Show {
            id,
            with_children,
            with_dependencies,
        } => handle_show(ctx, id, with_children, with_dependencies).await,
        TaskCommands::Update {
            id,
            title,
            description,
            status,
            priority,
        } => handle_update(ctx, id, title, description, status, priority).await,
        TaskCommands::Move { id, status } => handle_move(ctx, id, status).await,
        TaskCommands::Delete { id, yes } => handle_delete(ctx, id, yes).await,
    }
}

async fn handle_create(
    ctx: &Ctx,
    title: String,
    description: Option<String>,
    assignees: Vec<String>,
    parent: Option<String>,
    blocked_by: Vec<String>,
    priority: String,
    tags: Vec<String>,
) -> Result<()> {
    // Parse priority
    let priority: Priority = priority.parse()
        .map_err(|e: String| Error::Parse(e))?;

    // Parse parent task ID
    let parent_id = parent
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| Error::Parse(e.to_string()))?;

    // Parse blocked_by IDs
    let blocked_by_uuids: Vec<Uuid> = blocked_by
        .iter()
        .map(|s| Uuid::parse_str(s).map_err(|e| Error::Parse(e.to_string())))
        .collect::<Result<Vec<_>>>()?;

    // Parse assignees
    let assignee_list: Vec<Assignee> = assignees
        .iter()
        .map(|id| Assignee {
            id: id.clone(),
            kind: AssigneeType::Agent,
            role: AssigneeRole::Primary,
        })
        .collect();

    let input = CreateTask {
        title,
        description,
        priority,
        tags: if tags.is_empty() { None } else { Some(tags) },
        assignees: if assignee_list.is_empty() { None } else { Some(assignee_list) },
        blocked_by: if blocked_by_uuids.is_empty() { None } else { Some(blocked_by_uuids) },
        parent_id,
    };

    let repo = ctx.task_repo();
    let task = repo.create(&input).await?;

    println!("Created task: {}", task.id);
    println!("  Title: {}", task.title);
    println!("  Status: {}", task.status);
    println!("  Priority: {}", task.priority);

    Ok(())
}

async fn handle_list(
    ctx: &Ctx,
    statuses: Vec<String>,
    assignee: Option<String>,
    tag: Option<String>,
    parent: Option<String>,
    blocked: bool,
    format: String,
) -> Result<()> {
    let format: Format = format.parse()
        .map_err(|e: String| Error::Parse(e))?;

    // Parse status filters
    let status_filters: Vec<TaskStatus> = statuses
        .iter()
        .map(|s| s.parse())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e: String| Error::Parse(e))?;

    // Parse parent ID
    let parent_id = parent
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| Error::Parse(e.to_string()))?;

    let filters = TaskFilters {
        status: if status_filters.is_empty() { None } else { Some(status_filters) },
        assignee,
        tag,
        parent: parent_id,
        blocked: if blocked { Some(true) } else { None },
    };

    let repo = ctx.task_repo();
    let tasks = repo.find_all(&filters).await?;

    output::print_tasks(&tasks, format);

    Ok(())
}

async fn handle_show(
    ctx: &Ctx,
    id: String,
    with_children: bool,
    with_dependencies: bool,
) -> Result<()> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    let repo = ctx.task_repo();
    let task = repo.find_by_id(task_id).await?
        .ok_or_else(|| Error::TaskNotFound(task_id))?;

    output::print_task_detail(&task);

    if with_children {
        println!();
        println!("Subtasks:");
        let filters = TaskFilters {
            parent: Some(task_id),
            ..Default::default()
        };
        let children = repo.find_all(&filters).await?;
        if children.is_empty() {
            println!("  No subtasks");
        } else {
            for child in children {
                let id_str = child.id.to_string();
                let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
                println!("  [{}] {} - {}", child.status, id_short, child.title);
            }
        }
    }

    if with_dependencies {
        println!();
        if task.blocked_by.is_empty() {
            println!("Dependencies: None");
        } else {
            println!("Blocked by:");
            for dep_id in &task.blocked_by {
                match repo.find_by_id(*dep_id).await? {
                    Some(dep) => {
                        let dep_id_str = dep_id.to_string();
                        let dep_id_short = if dep_id_str.len() >= 8 { &dep_id_str[..8] } else { &dep_id_str };
                        println!("  [{}] {} - {}", dep.status, dep_id_short, dep.title);
                    },
                    None => {
                        let dep_id_str = dep_id.to_string();
                        let dep_id_short = if dep_id_str.len() >= 8 { &dep_id_str[..8] } else { &dep_id_str };
                        println!("  {} (not found)", dep_id_short);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_update(
    ctx: &Ctx,
    id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
) -> Result<()> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    let status = status
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: String| Error::Parse(e))?;

    let priority = priority
        .map(|p| p.parse())
        .transpose()
        .map_err(|e: String| Error::Parse(e))?;

    let input = UpdateTask {
        title,
        description,
        status,
        priority,
        tags: None,
    };

    let repo = ctx.task_repo();
    let task = repo.update(task_id, &input).await?;

    println!("Updated task: {}", task.id);
    println!("  Status: {}", task.status);
    println!("  Priority: {}", task.priority);

    Ok(())
}

async fn handle_move(ctx: &Ctx, id: String, status: String) -> Result<()> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    let new_status: TaskStatus = status.parse()
        .map_err(|e: String| Error::Parse(e))?;

    let input = UpdateTask {
        status: Some(new_status),
        ..Default::default()
    };

    let repo = ctx.task_repo();
    let task = repo.update(task_id, &input).await?;

    let id_str = id.to_string();
    let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
    println!("Moved task {} to {}", id_short, task.status);

    Ok(())
}

async fn handle_delete(ctx: &Ctx, id: String, yes: bool) -> Result<()> {
    let task_id = Uuid::parse_str(&id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    if !yes {
        // Show task first
        let repo = ctx.task_repo();
        let task = repo.find_by_id(task_id).await?
            .ok_or_else(|| Error::TaskNotFound(task_id))?;

        println!("About to delete task:");
        println!("  ID: {}", task.id);
        println!("  Title: {}", task.title);
        println!("  Status: {}", task.status);
        println!();
        print!("Are you sure? (y/N): ");
        std::io::stdout().flush().map_err(|e| Error::Io(e))?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| Error::Io(e))?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let repo = ctx.task_repo();
    repo.delete(task_id).await?;

    let id_str = id.to_string();
    let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
    println!("Deleted task: {}", id_short);

    Ok(())
}
