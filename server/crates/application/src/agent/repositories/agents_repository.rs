use std::sync::Arc;

use async_trait::async_trait;
use domain::agent::agent::Agent;

use crate::net::repositories::agent_connection::AgentConnection;

#[async_trait]
pub trait AgentsRepository: Send + Sync {
    async fn ensure_agents(&self) -> Result<(), String>;
    async fn set_agents_net(&self, net_agent: Arc<dyn AgentConnection>) -> Result<(), String>;
    async fn get_agent_net(&self, id: String) -> Option<Arc<dyn AgentConnection>>;
    async fn get_agent_from_id(&self, id: String) -> Option<Agent>;
}
