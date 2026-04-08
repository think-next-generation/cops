use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cops")]
#[command(about = "Company Operations Task System", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to config file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Task management
    #[command(subcommand)]
    Task(TaskCommands),

    /// Board / Kanban view
    #[command(subcommand)]
    Board(BoardCommands),

    /// Question management
    #[command(subcommand)]
    Question(QuestionCommands),

    /// Comment management
    #[command(subcommand)]
    Comment(CommentCommands),

    /// System status and monitoring
    #[command(subcommand)]
    Status(StatusCommands),

    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Database management
    #[command(subcommand)]
    Db(DbCommands),

    /// Start web server
    Web(WebCommands),
}

#[derive(Subcommand)]
pub enum TaskCommands {
    /// Create a new task
    Create {
        /// Task title
        title: String,
        /// Description (markdown)
        #[arg(short, long)]
        description: Option<String>,
        /// Assign to agent/human (repeatable)
        #[arg(short = 'a', long)]
        assignee: Vec<String>,
        /// Parent task ID for subtask
        #[arg(short, long)]
        parent: Option<String>,
        /// Blocking dependency task ID (repeatable)
        #[arg(long)]
        blocked_by: Vec<String>,
        /// Priority: low, medium, high, urgent
        #[arg(short = 'P', long, default_value = "medium")]
        priority: String,
        /// Tag (repeatable)
        #[arg(short, long)]
        tag: Vec<String>,
    },

    /// List tasks
    List {
        /// Filter by status (repeatable)
        #[arg(short, long)]
        status: Vec<String>,
        /// Filter by assignee
        #[arg(short, long)]
        assignee: Option<String>,
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Filter by parent task
        #[arg(short, long)]
        parent: Option<String>,
        /// Show only blocked tasks
        #[arg(long)]
        blocked: bool,
        /// Output format: table, json, simple
        #[arg(short = 'F', long, default_value = "table")]
        format: String,
    },

    /// Show task details
    Show {
        /// Task ID
        id: String,
        /// Include subtasks
        #[arg(long)]
        with_children: bool,
        /// Show dependency chain
        #[arg(long)]
        with_dependencies: bool,
    },

    /// Update a task
    Update {
        /// Task ID
        id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// New status
        #[arg(short, long)]
        status: Option<String>,
        /// New priority
        #[arg(long)]
        priority: Option<String>,
    },

    /// Move task to new status (quick shortcut)
    Move {
        /// Task ID
        id: String,
        /// New status
        status: String,
    },

    /// Delete a task
    Delete {
        /// Task ID
        id: String,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum BoardCommands {
    /// Show kanban board
    Show {
        /// Filter by status
        #[arg(long)]
        filter: Option<String>,
        /// Filter by assignee
        #[arg(short, long)]
        assignee: Option<String>,
        /// Auto-refresh every 5 seconds
        #[arg(short, long)]
        watch: bool,
    },
}

#[derive(Subcommand)]
pub enum QuestionCommands {
    /// Create a question on a task
    Create {
        /// Task ID
        task_id: String,
        /// Question text
        question: String,
        /// Question type: open, single, multi
        #[arg(short = 't', long, default_value = "open")]
        qtype: String,
        /// Options for choice types (repeatable)
        #[arg(short, long)]
        option: Vec<String>,
        /// Urgency: low, normal, high
        #[arg(short, long, default_value = "normal")]
        urgency: String,
    },

    /// List questions
    List {
        /// Filter by task
        #[arg(short, long)]
        task: Option<String>,
        /// Only unanswered
        #[arg(long)]
        unanswered: bool,
        /// Only urgent
        #[arg(long)]
        urgent: bool,
    },

    /// Answer a question
    Answer {
        /// Question ID
        id: String,
        /// Answer text
        answer: String,
        /// Answered by
        #[arg(long)]
        by: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CommentCommands {
    /// Add a comment
    Add {
        /// Task ID
        task_id: String,
        /// Comment message
        message: String,
        /// Author ID
        #[arg(short, long)]
        author: Option<String>,
    },

    /// List comments on a task
    List {
        /// Task ID
        task_id: String,
        /// Newest first
        #[arg(long)]
        reverse: bool,
    },
}

#[derive(Subcommand)]
pub enum StatusCommands {
    /// Show system status
    #[command(name = "status")]
    Show {
        /// Output format: json, simple
        #[arg(short = 'F', long, default_value = "simple")]
        format: String,
        /// Only changes since timestamp
        #[arg(long)]
        since: Option<String>,
    },

    /// Watch for changes (continuous stream)
    Watch,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize config file
    Init,

    /// Show current configuration
    Show,
}

#[derive(Subcommand)]
pub enum DbCommands {
    /// Run database migrations
    Migrate,

    /// Show migration status
    Status,
}

#[derive(Args)]
pub struct WebCommands {
    /// Port number
    #[arg(short, long, default_value = "9090")]
    pub port: u16,

    /// Bind host
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Open browser on start
    #[arg(long)]
    pub open: bool,

    /// API only (no SPA frontend)
    #[arg(long)]
    pub no_ui: bool,
}
