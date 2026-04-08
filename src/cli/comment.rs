//! Comment command handlers
//!
//! Free-form thread discussions for tasks.

use super::args::CommentCommands;
use super::ctx::Ctx;
use super::output;
use crate::core::{CreateComment, AuthorType};
use crate::error::{Error, Result};
use uuid::Uuid;

pub async fn handle(cmd: CommentCommands, ctx: &Ctx) -> Result<()> {
    match cmd {
        CommentCommands::Add {
            task_id,
            message,
            author,
        } => handle_add(ctx, task_id, message, author).await,
        CommentCommands::List {
            task_id,
            reverse,
        } => handle_list(ctx, task_id, reverse).await,
    }
}

async fn handle_add(
    ctx: &Ctx,
    task_id: String,
    content: String,
    author: Option<String>,
) -> Result<()> {
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    // Verify task exists
    let task_repo = ctx.task_repo();
    task_repo.find_by_id(task_uuid).await?
        .ok_or_else(|| Error::Custom(format!("Task not found: {}", task_id)))?;

    let input = CreateComment {
        content,
        author_id: author,
        author_type: Some(AuthorType::Human),
    };

    let repo = ctx.comment_repo();
    let comment = repo.create(task_uuid, &input).await?;

    let task_id_str = task_id.to_string();
    let task_id_short = if task_id_str.len() >= 8 { &task_id_str[..8] } else { &task_id_str };
    println!("Added comment to task {}", task_id_short);
    println!("  Comment ID: {}", comment.id);

    Ok(())
}

async fn handle_list(
    ctx: &Ctx,
    task_id: String,
    reverse: bool,
) -> Result<()> {
    let task_uuid = Uuid::parse_str(&task_id)
        .map_err(|e| Error::Parse(e.to_string()))?;

    let repo = ctx.comment_repo();
    let mut comments = repo.find_by_task(task_uuid).await?;

    if reverse {
        comments.reverse();
    }

    let task_id_str = task_id.to_string();
    let task_id_short = if task_id_str.len() >= 8 { &task_id_str[..8] } else { &task_id_str };
    println!("Comments on task {}:", task_id_short);
    println!();
    output::print_comments(&comments);

    Ok(())
}
