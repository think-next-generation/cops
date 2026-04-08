//! CLI module - command-line interface handling

mod args;
mod board;
mod comment;
mod config_cmd;
mod ctx;
mod db_cmd;
mod output;
mod question;
mod status;
mod task;
mod web;

use clap::Parser;

pub use args::*;

pub fn run() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        // Initialize context with config and database
        let ctx = match ctx::Ctx::init(args.config.as_ref()).await {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Error initializing: {}", e);
                std::process::exit(1);
            }
        };

        match args.command {
            Commands::Task(cmd) => task::handle(cmd, &ctx).await,
            Commands::Board(cmd) => board::handle(cmd, &ctx).await,
            Commands::Question(cmd) => question::handle(cmd, &ctx).await,
            Commands::Comment(cmd) => comment::handle(cmd, &ctx).await,
            Commands::Status(cmd) => status::handle(cmd, &ctx).await,
            Commands::Config(cmd) => config_cmd::handle(cmd, &ctx).await,
            Commands::Db(cmd) => db_cmd::handle(cmd, &ctx).await,
            Commands::Web(cmd) => web::handle(cmd, &ctx).await,
        }
    })?;

    Ok(())
}
