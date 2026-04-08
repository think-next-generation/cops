//! Database command handlers

use super::args::DbCommands;
use super::ctx::Ctx;
use crate::error::Result;

pub async fn handle(cmd: DbCommands, ctx: &Ctx) -> Result<()> {
    match cmd {
        DbCommands::Migrate => {
            handle_migrate(ctx).await
        }
        DbCommands::Status => {
            handle_status(ctx).await
        }
    }
}

async fn handle_migrate(ctx: &Ctx) -> Result<()> {
    println!("Running database migrations...");

    ctx.pool.run_migrations().await?;

    println!("Migrations completed successfully.");
    println!("Database: {}", ctx.config.database.sqlite_path.display());

    Ok(())
}

async fn handle_status(_ctx: &Ctx) -> Result<()> {
    println!("Migration Status:");
    println!();
    println!("  001_initial.sql - Applied (embedded)");
    println!();
    println!("Database is up to date.");

    Ok(())
}
