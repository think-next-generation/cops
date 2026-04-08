//! Database pool abstraction supporting SQLite and MySQL
//!
//! Provides a unified interface for both SQLite (default) and MySQL/MariaDB
//! backends with automatic migration support.

use sqlx::{MySqlPool, SqlitePool};
use std::path::Path;

use crate::config::Config;
use crate::error::Result;

/// Database pool abstraction supporting SQLite and MySQL
#[derive(Clone)]
pub enum DbPool {
    Sqlite(SqlitePool),
    MySql(MySqlPool),
}

impl DbPool {
    /// Connect to database using configuration
    pub async fn connect(config: &Config) -> Result<Self> {
        let url = config.database_url();

        if config.database.backend == "mariadb" {
            let pool = MySqlPool::connect(&url).await?;
            Ok(Self::MySql(pool))
        } else {
            // Ensure parent directory exists for SQLite
            if let Some(parent) = Path::new(&config.database.sqlite_path).parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            let pool = SqlitePool::connect(&url).await?;
            Ok(Self::Sqlite(pool))
        }
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        let migration_sql = include_str!("migrations/001_initial.sql");

        match self {
            Self::Sqlite(pool) => {
                sqlx::raw_sql(migration_sql).execute(pool).await?;
            }
            Self::MySql(pool) => {
                // MySQL needs slightly different syntax
                let mysql_sql = migration_sql
                    .replace("TEXT PRIMARY KEY", "VARCHAR(36) PRIMARY KEY")
                    .replace("AUTOINCREMENT", "AUTO_INCREMENT");
                for stmt in mysql_sql.split(';').filter(|s| !s.trim().is_empty()) {
                    sqlx::query(&format!("{};", stmt)).execute(pool).await?;
                }
            }
        }
        Ok(())
    }

    /// Check if pool is SQLite
    pub fn is_sqlite(&self) -> bool {
        matches!(self, Self::Sqlite(_))
    }

    /// Check if pool is MySQL/MariaDB
    pub fn is_mysql(&self) -> bool {
        matches!(self, Self::MySql(_))
    }
}

// Convenience methods for executing queries
impl DbPool {
    /// Get SQLite pool (panics if not SQLite)
    pub fn as_sqlite(&self) -> &SqlitePool {
        match self {
            Self::Sqlite(pool) => pool,
            _ => panic!("Not a SQLite pool"),
        }
    }

    /// Get MySQL pool (panics if not MySQL)
    pub fn as_mysql(&self) -> &MySqlPool {
        match self {
            Self::MySql(pool) => pool,
            _ => panic!("Not a MySQL pool"),
        }
    }
}
