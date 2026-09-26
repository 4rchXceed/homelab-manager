use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{config::Config, db::database::Database};

#[derive(Clone)]
pub struct AgentContext {
    pub db: Arc<RwLock<Database>>,
    pub config: Arc<RwLock<Config>>,
}

impl AgentContext {
    pub fn new(db: Database, config: Config) -> Self {
        return Self {
            db: Arc::new(RwLock::new(db)),
            config: Arc::new(RwLock::new(config)),
        };
    }
}
