//! Output formatting utilities
//!
//! Provides table, JSON, and simple output formats for CLI commands.

use chrono::{DateTime, Utc};
use comfy_table::{Cell, Color, Table, presets::UTF8_FULL};
use crate::core::{Task, TaskStatus, Priority, Question, Comment};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Table,
    Json,
    Simple,
}

impl std::str::FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "simple" => Ok(Self::Simple),
            _ => Err(format!("Invalid format: {}. Use table, json, or simple.", s)),
        }
    }
}

/// Format timestamp for display
pub fn format_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

/// Get color for task status
fn status_color(status: TaskStatus) -> Color {
    match status {
        TaskStatus::New => Color::Cyan,
        TaskStatus::Assigned => Color::Blue,
        TaskStatus::InProgress => Color::Yellow,
        TaskStatus::Blocked => Color::Red,
        TaskStatus::Waiting => Color::Magenta,
        TaskStatus::Review => Color::DarkYellow,
        TaskStatus::Done => Color::Green,
        TaskStatus::Archived => Color::DarkGrey,
    }
}

/// Get color for priority
fn priority_color(priority: Priority) -> Color {
    match priority {
        Priority::Low => Color::DarkGrey,
        Priority::Medium => Color::Yellow,
        Priority::High => Color::Red,
        Priority::Urgent => Color::Magenta,
    }
}

/// Print tasks in the specified format
pub fn print_tasks(tasks: &[Task], format: Format) {
    match format {
        Format::Table => print_tasks_table(tasks),
        Format::Json => print_tasks_json(tasks),
        Format::Simple => print_tasks_simple(tasks),
    }
}

fn print_tasks_table(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Title", "Status", "Priority", "Assignees", "Updated"]);

    for task in tasks {
        let id_str = task.id.to_string();
        let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
        let assignees: String = task.assignees.iter()
            .map(|a| a.id.as_str())
            .take(2)
            .collect::<Vec<_>>()
            .join(", ");
        let assignees = if task.assignees.len() > 2 {
            format!("{}...", assignees)
        } else if assignees.is_empty() {
            "-".to_string()
        } else {
            assignees
        };

        table.add_row(vec![
            Cell::new(id_short),
            Cell::new(&truncate(&task.title, 30)),
            Cell::new(task.status.to_string()).fg(status_color(task.status)),
            Cell::new(task.priority.to_string()).fg(priority_color(task.priority)),
            Cell::new(assignees),
            Cell::new(format_time(&task.updated_at)),
        ]);
    }

    println!("{table}");
}

fn print_tasks_json(tasks: &[Task]) {
    match serde_json::to_string_pretty(&tasks) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing tasks: {}", e),
    }
}

fn print_tasks_simple(tasks: &[Task]) {
    for task in tasks {
        let id_str = task.id.to_string();
        let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
        println!("[{}] {} {}: {}",
            task.status.to_string(),
            id_short,
            task.priority.to_string(),
            task.title
        );
    }
}

/// Print a single task in detail
pub fn print_task_detail(task: &Task) {
    println!("Task: {}", task.id);
    println!("  Title: {}", task.title);
    if let Some(ref desc) = task.description {
        println!("  Description: {}", desc);
    }
    println!("  Status: {}", task.status);
    println!("  Priority: {}", task.priority);
    if !task.tags.is_empty() {
        println!("  Tags: {}", task.tags.join(", "));
    }
    if !task.assignees.is_empty() {
        let assignees: Vec<String> = task.assignees.iter()
            .map(|a| format!("{} ({:?})", a.id, a.role))
            .collect();
        println!("  Assignees: {}", assignees.join(", "));
    }
    if !task.blocked_by.is_empty() {
        let blocked: Vec<String> = task.blocked_by.iter()
            .map(|id| {
                let id_str = id.to_string();
                if id_str.len() >= 8 { id_str[..8].to_string() } else { id_str }
            })
            .collect();
        println!("  Blocked by: {}", blocked.join(", "));
    }
    if let Some(parent) = task.parent_id {
        println!("  Parent: {}", parent);
    }
    println!("  Created: {}", task.created_at.format("%Y-%m-%d %H:%M UTC"));
    println!("  Updated: {}", task.updated_at.format("%Y-%m-%d %H:%M UTC"));
}

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Print questions in the specified format
pub fn print_questions(questions: &[Question], format: Format) {
    match format {
        Format::Table => print_questions_table(questions),
        Format::Json => {
            match serde_json::to_string_pretty(&questions) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("Error serializing questions: {}", e),
            }
        }
        Format::Simple => {
            for q in questions {
                let id_str = q.id.to_string();
                let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
                let status = if q.is_answered() { "✓" } else { "?" };
                println!("[{}] {} {}", status, id_short, truncate(&q.question_text, 50));
            }
        }
    }
}

fn print_questions_table(questions: &[Question]) {
    if questions.is_empty() {
        println!("No questions found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Task", "Question", "Type", "Urgency", "Status"]);

    for q in questions {
        let id_str = q.id.to_string();
        let id_short = if id_str.len() >= 8 { &id_str[..8] } else { &id_str };
        let task_str = q.task_id.to_string();
        let task_short = if task_str.len() >= 8 { &task_str[..8] } else { &task_str };
        let status = if q.is_answered() { "Answered" } else { "Pending" };

        table.add_row(vec![
            Cell::new(id_short),
            Cell::new(task_short),
            Cell::new(truncate(&q.question_text, 30)),
            Cell::new(format!("{:?}", q.question_type)),
            Cell::new(format!("{:?}", q.urgency)),
            Cell::new(status),
        ]);
    }

    println!("{table}");
}

/// Print comments
pub fn print_comments(comments: &[Comment]) {
    if comments.is_empty() {
        println!("No comments.");
        return;
    }

    for comment in comments {
        println!("[{}] {} ({:?}): {}",
            format_time(&comment.created_at),
            comment.author_id,
            comment.author_type,
            comment.content
        );
    }
}
