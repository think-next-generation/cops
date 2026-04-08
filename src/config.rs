//! Configuration module - application settings
//!
//! Supports SQLite (default) or MariaDB database backend,
//! server configuration, and board configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub board: BoardConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_backend")]
    pub backend: String,
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: PathBuf,
    pub mariadb: Option<MariaDbConfig>,
}

fn default_db_backend() -> String {
    "sqlite".to_string()
}

fn default_sqlite_path() -> PathBuf {
    PathBuf::from("./data/cops.db")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MariaDbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_ws_enabled")]
    pub websocket_enabled: bool,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    9090
}

fn default_ws_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoardConfig {
    #[serde(default = "default_columns")]
    pub default_columns: Vec<String>,
    #[serde(default = "default_hidden")]
    pub hidden_columns: Vec<String>,
}

fn default_columns() -> Vec<String> {
    vec![
        "NEW".to_string(),
        "ASSIGNED".to_string(),
        "IN_PROGRESS".to_string(),
        "BLOCKED".to_string(),
        "REVIEW".to_string(),
        "DONE".to_string(),
    ]
}

fn default_hidden() -> Vec<String> {
    vec!["ARCHIVED".to_string(), "WAITING".to_string()]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            board: BoardConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            backend: default_db_backend(),
            sqlite_path: default_sqlite_path(),
            mariadb: None,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            websocket_enabled: default_ws_enabled(),
        }
    }
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            default_columns: default_columns(),
            hidden_columns: default_hidden(),
        }
    }
}

impl Config {
    /// Load configuration from default search paths
    ///
    /// Search order:
    /// 1. $COPS_CONFIG environment variable
    /// 2. ./cops.toml (current directory)
    /// 3. ~/.config/cops/cops.toml (user config)
    /// 4. /etc/cops/cops.toml (system config)
    pub fn load() -> crate::Result<Self> {
        let paths = Self::config_paths();

        for path in paths {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let config: Config = toml::from_str(&content)?;
                return Ok(config);
            }
        }

        Ok(Self::default())
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &PathBuf) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    fn config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Environment variable
        if let Ok(p) = std::env::var("COPS_CONFIG") {
            paths.push(PathBuf::from(p));
        }

        // 2. Current directory
        paths.push(PathBuf::from("./cops.toml"));

        // 3. User config directory
        paths.push(
            dirs::config_dir()
                .map(|p| p.join("cops/cops.toml"))
                .unwrap_or_default(),
        );

        // 4. System config
        paths.push(PathBuf::from("/etc/cops/cops.toml"));

        paths
    }

    /// Get the database connection URL
    ///
    /// Returns SQLite URL or MariaDB/MySQL connection string
    pub fn database_url(&self) -> String {
        if self.database.backend == "mariadb" {
            if let Some(ref mdb) = self.database.mariadb {
                return format!(
                    "mysql://{}:{}@{}:{}/{}",
                    mdb.username, mdb.password, mdb.host, mdb.port, mdb.database
                );
            }
        }
        format!(
            "sqlite:{}?mode=rwc",
            self.database.sqlite_path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database.backend, "sqlite");
        assert_eq!(config.server.port, 9090);
        assert!(config.server.websocket_enabled);
        assert_eq!(config.board.default_columns.len(), 6);
    }

    #[test]
    fn test_database_url_sqlite() {
        let config = Config::default();
        let url = config.database_url();
        assert!(url.starts_with("sqlite:"));
        assert!(url.contains("cops.db"));
    }

    #[test]
    fn test_database_url_mariadb() {
        let mut config = Config::default();
        config.database.backend = "mariadb".to_string();
        config.database.mariadb = Some(MariaDbConfig {
            host: "localhost".to_string(),
            port: 3306,
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
        });

        let url = config.database_url();
        assert!(url.starts_with("mysql://"));
        assert!(url.contains("localhost:3306"));
    }

    #[test]
    fn test_deserialize_config() {
        let toml_str = r#"
[database]
backend = "sqlite"
sqlite_path = "/tmp/test.db"

[server]
port = 8080
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.database.sqlite_path, PathBuf::from("/tmp/test.db"));
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "127.0.0.1"); // default
    }
}
