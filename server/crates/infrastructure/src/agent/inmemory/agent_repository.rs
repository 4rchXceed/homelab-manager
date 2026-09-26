use std::sync::Arc;

use application::{
    agent::repositories::agents_repository::AgentsRepository,
    database::repositories::agents_db::AgentsDb,
    net::repositories::agent_connection::AgentConnection,
};
use async_trait::async_trait;
use config::agent::config::AgentConfig;
use domain::agent::agent::Agent;
use tokio::sync::RwLock;

use crate::agent::inmemory::agent::InMemoryAgent;

#[derive(Clone)]
pub struct InMemoryAgentRepository {
    agents: Arc<RwLock<Vec<InMemoryAgent>>>,
    agents_db: Arc<dyn AgentsDb>,
}

impl InMemoryAgentRepository {
    pub fn new(agents_db: Arc<dyn AgentsDb>, agents_config: Vec<AgentConfig>) -> Self {
        let agents = agents_config
            .into_iter()
            .map(|config| InMemoryAgent::new(config))
            .collect();

        let agents_locked = Arc::new(RwLock::new(agents));

        return Self {
            agents: agents_locked,
            agents_db,
        };
    }
}

#[async_trait]
impl AgentsRepository for InMemoryAgentRepository {
    async fn ensure_agents(&self) -> Result<(), String> {
        for agent in self.agents.write().await.iter_mut() {
            let reverse_api_key = self.agents_db.ensure_agent(agent.id.clone()).await?;

            agent.agent = Some(Agent {
                api_key: agent.config.api_key.clone(),
                id: agent.config.id.clone(),
                ip: agent.config.ip.to_string(),
                reverse_api_key: reverse_api_key,
                storages: Vec::new(),
                storages_configs: agent.config.storages.clone(),
            })
        }

        return Ok(());
    }

    async fn set_agents_net(&self, net_agent: Arc<dyn AgentConnection>) -> Result<(), String> {
        for agent in self.agents.write().await.iter_mut() {
            agent.net_agent = Some(net_agent.clone());
        }

        return Ok(());
    }

    async fn get_agent_from_id(&self, id: String) -> Option<Agent> {
        let agents = self.agents.read().await;
        let agent = agents.iter().find(|agent| agent.config.id == id)?;

        return agent.agent.clone();
    }

    async fn get_agent_net(&self, id: String) -> Option<Arc<dyn AgentConnection>> {
        let agents = self.agents.read().await;
        let agent = agents.iter().find(|agent| agent.config.id == id)?;

        return agent.net_agent.clone();
    }
}
