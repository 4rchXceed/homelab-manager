use std::sync::Arc;

use config::agent::config::AgentConfig;
use domain::agent::update_agent::UpdateAgent;

use crate::database::repositories::agents_db::AgentsDb;

pub struct EnsureAgentInDb {
    agents_db: Arc<dyn AgentsDb>,
}

impl EnsureAgentInDb {
    pub fn new(agents_db: Arc<dyn AgentsDb>) -> Self {
        return Self { agents_db };
    }

    pub async fn ensure_agent(&self, agent_config: &AgentConfig) -> Result<(), String> {
        self.agents_db
            .ensure_agent(UpdateAgent {
                api_key: agent_config.api_key.clone(),
                id: agent_config.id.clone(),
                ip: agent_config.ip.to_string(),
            })
            .await?;

        return Ok(());
    }

    pub async fn ensure_agents(&self, agent_configs: &Vec<AgentConfig>) -> Result<(), String> {
        for agent_config in agent_configs {
            self.ensure_agent(agent_config).await?;
        }

        return Ok(());
    }
}
