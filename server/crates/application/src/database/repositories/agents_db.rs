use async_trait::async_trait;
use domain::agent::{agent::Agent, update_agent::UpdateAgent};

#[async_trait]
pub trait AgentsDb {
    async fn get_agent_by_id(&self, id: String) -> Result<Option<Agent>, String>;

    async fn ensure_agent(&self, agent: UpdateAgent) -> Result<(), String>;
}
