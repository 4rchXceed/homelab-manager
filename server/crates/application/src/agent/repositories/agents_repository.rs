use std::sync::Arc;

use async_trait::async_trait;
use domain::agent::agent::Agent;

use crate::net::repositories::net_connection::NetworkConnection;

#[async_trait]
pub trait AgentsRepository: Send + Sync {
    async fn ensure_agents(&self) -> Result<(), String>;
    async fn set_agents_net(&self, net_agent: Arc<dyn NetworkConnection>) -> Result<(), String>;
    async fn get_agent_from_id(&self, id: String) -> Option<Agent>;
}
