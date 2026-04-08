//! Shared API state

use std::sync::Arc;
use crate::config::Config;
use crate::db::DbPool;

/// Shared state for API handlers
#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<Config>,
    pub pool: Arc<DbPool>,
}

impl ApiState {
    pub fn new(config: Config, pool: DbPool) -> Self {
        Self {
            config: Arc::new(config),
            pool: Arc::new(pool),
        }
    }
}
