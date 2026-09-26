use std::sync::Arc;

use application::net::repositories::net_connection::NetworkConnection;
use config::agent::config::AgentConfig;
use domain::agent::agent::Agent;

#[derive(Clone)]
pub struct InMemoryAgent {
    pub id: String,
    pub agent: Option<Agent>,
    pub config: AgentConfig,
    pub net_agent: Option<Arc<dyn NetworkConnection>>,
}

impl InMemoryAgent {
    pub fn new(config: AgentConfig) -> Self {
        return Self {
            id: config.id.clone(),
            agent: None,
            config,
            net_agent: None,
        };
    }
}
