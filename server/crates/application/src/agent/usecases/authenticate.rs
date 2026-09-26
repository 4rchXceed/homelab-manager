use std::sync::Arc;

use domain::agent::agent::Agent;
use log::trace;

use crate::{
    agent::{errors::AgentCommunicationError, repositories::agents_repository::AgentsRepository},
    net::repositories::net_connection::NetworkConnection,
};

pub struct AuthenticateAgent {
    agent_repo: Arc<dyn AgentsRepository>,
}

impl AuthenticateAgent {
    pub fn new(agent_repo: Arc<dyn AgentsRepository>) -> Self {
        return Self { agent_repo };
    }

    async fn get_db_agent(
        &self,
        net_agent: Arc<dyn NetworkConnection>,
    ) -> Result<Agent, AgentCommunicationError> {
        trace!("Receiving agent's ID from network...");

        let agent_id_bytes = net_agent
            .recv_until(b'\n')
            .await
            .map_err(|e| AgentCommunicationError::NetError(e))?;

        let agent_id = String::from_utf8(agent_id_bytes)
            .map_err(|e| AgentCommunicationError::InvalidAgentIdUtf8(e))?
            .trim()
            .to_string();

        trace!("Looking for agent with ID: {agent_id}");

        let agent = self
            .agent_repo
            .get_agent_from_id(agent_id.clone())
            .await
            .ok_or(AgentCommunicationError::AgentWithIdNotFound(agent_id))?;

        return Ok(agent);
    }

    pub async fn authenticate(
        &self,
        net_agent: Arc<dyn NetworkConnection>,
    ) -> Result<Agent, AgentCommunicationError> {
        trace!("Trying to authenticate agent...");

        let result = self.unsafe_authenticate(net_agent.clone()).await;

        if let Err(e) = result {
            trace!("Authentication failed: {e}");

            net_agent
                .close()
                .await
                .map_err(|e| AgentCommunicationError::NetError(e))?;

            return Err(e);
        }

        self.agent_repo
            .set_agents_net(net_agent.clone())
            .await
            .map_err(|e| AgentCommunicationError::FailedToSetAgentNetConnection(e))?;

        return result;
    }

    async fn check_agent_api_key(
        &self,
        agent: &Agent,
        net_agent: Arc<dyn NetworkConnection>,
    ) -> Result<(), AgentCommunicationError> {
        trace!("Receiving agent's API key...");

        let response = net_agent
            .recv_until(b'\n')
            .await
            .map_err(|e| AgentCommunicationError::NetError(e))?;

        let response_str = String::from_utf8(response)
            .map_err(|e| AgentCommunicationError::InvalidApiKeyUtf8(e))?;

        if response_str.trim() != agent.api_key {
            trace!("Agent API key mismatch."); // Not showing the api keys for obvious reasons

            net_agent
                .send_raw(b"ER".to_vec())
                .await
                .map_err(|e| AgentCommunicationError::NetError(e))?;

            return Err(AgentCommunicationError::AuthRejected);
        }

        trace!("Agent API key is correct!");

        net_agent
            .send_raw(b"OK".to_vec())
            .await
            .map_err(|e| AgentCommunicationError::NetError(e))?;

        return Ok(());
    }

    async fn process_reverse_api_key(
        &self,
        agent: &Agent,
        net_agent: Arc<dyn NetworkConnection>,
    ) -> Result<(), AgentCommunicationError> {
        trace!("Sending agent's reverse API key...");

        net_agent
            .send_raw(agent.reverse_api_key.clone().into_bytes())
            .await
            .map_err(|e| AgentCommunicationError::NetError(e))?;

        let ack = net_agent
            .recv_raw(2)
            .await
            .map_err(|e| AgentCommunicationError::NetError(e))?;

        if ack != b"OK" {
            trace!("Agent did not acknowledge reverse API key :(");

            return Err(AgentCommunicationError::ReverseApiKeyNotAcknowledged);
        }

        trace!("Agent acknowledged reverse API key :)");

        return Ok(());
    }

    async fn unsafe_authenticate(
        &self,
        net_agent: Arc<dyn NetworkConnection>,
    ) -> Result<Agent, AgentCommunicationError> {
        trace!("Receiving agent's ID...");

        let agent = self.get_db_agent(net_agent.clone()).await?;

        trace!("Sending auth ack...");

        self.first_ack(net_agent.clone()).await?;

        trace!("Checking agent's API key...");

        self.check_agent_api_key(&agent, net_agent.clone()).await?;

        trace!("Sending agent's reverse API key...");

        self.process_reverse_api_key(&agent, net_agent).await?;

        return Ok(agent);
    }

    async fn first_ack(
        &self,
        net_agent: Arc<dyn NetworkConnection>,
    ) -> Result<(), AgentCommunicationError> {
        net_agent
            .send_raw(b"AUTH".to_vec())
            .await
            .map_err(|e| AgentCommunicationError::NetError(e))?;
        Ok(())
    }
}
