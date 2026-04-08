//! Config command handlers

use super::args::ConfigCommands;
use super::ctx::Ctx;
use crate::error::Result;

pub async fn handle(cmd: ConfigCommands, ctx: &Ctx) -> Result<()> {
    match cmd {
        ConfigCommands::Init => {
            handle_init().await
        }
        ConfigCommands::Show => {
            handle_show(ctx).await
        }
    }
}

async fn handle_init() -> Result<()> {
    let config_path = std::path::PathBuf::from("./cops.toml");

    if config_path.exists() {
        eprintln!("Config file already exists: ./cops.toml");
        eprintln!("Use 'cops config show' to view current configuration.");
        return Ok(());
    }

    let config_content = r#"# cops.toml - Task System Configuration

[database]
backend = "sqlite"
sqlite_path = "./data/cops.db"

# MariaDB configuration (uncomment to use)
# [database.mariadb]
# host = "127.0.0.1"
# port = 3306
# database = "company_ops"
# username = "company_ops"
# password = "company_opsPassword"

[server]
host = "127.0.0.1"
port = 9090
websocket_enabled = true

[board]
default_columns = ["NEW", "ASSIGNED", "IN_PROGRESS", "BLOCKED", "REVIEW", "DONE"]
hidden_columns = ["ARCHIVED", "WAITING"]
"#;

    std::fs::write(&config_path, config_content)?;

    // Create data directory
    std::fs::create_dir_all("./data")?;

    println!("Created: ./cops.toml");
    println!("Created: ./data/ directory");
    println!();
    println!("Edit cops.toml to configure database and agents.");
    println!("Run 'cops db migrate' to initialize the database.");

    Ok(())
}

async fn handle_show(ctx: &Ctx) -> Result<()> {
    println!("Configuration:");
    println!();
    println!("[database]");
    println!("  backend: {}", ctx.config.database.backend);
    println!("  sqlite_path: {}", ctx.config.database.sqlite_path.display());
    if let Some(ref mdb) = ctx.config.database.mariadb {
        println!("  mariadb:");
        println!("    host: {}", mdb.host);
        println!("    port: {}", mdb.port);
        println!("    database: {}", mdb.database);
        println!("    username: {}", mdb.username);
    }
    println!();
    println!("[server]");
    println!("  host: {}", ctx.config.server.host);
    println!("  port: {}", ctx.config.server.port);
    println!("  websocket_enabled: {}", ctx.config.server.websocket_enabled);
    println!();
    println!("[board]");
    println!("  default_columns: {:?}", ctx.config.board.default_columns);
    println!("  hidden_columns: {:?}", ctx.config.board.hidden_columns);

    Ok(())
}
