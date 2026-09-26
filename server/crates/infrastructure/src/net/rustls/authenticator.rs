use std::sync::Arc;

use application::agent::repositories::agents_repository::AgentsRepository;
use domain::agent::agent::Agent;
use log::trace;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;

use crate::net::rustls::errors::AgentCommunicationError;

pub struct TlsAgentAuthenticator {
    agent_repo: Arc<dyn AgentsRepository>,
}

impl TlsAgentAuthenticator {
    pub fn new(agent_repo: Arc<dyn AgentsRepository>) -> Self {
        return Self { agent_repo };
    }

    async fn get_db_agent(
        &self,
        net_agent: &mut TlsStream<TcpStream>,
    ) -> Result<Agent, AgentCommunicationError> {
        trace!("Receiving agent's ID from network...");

        let mut agent_id_bytes = Vec::new();
        net_agent
            .read_until(b'\n', &mut agent_id_bytes)
            .await
            .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;

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
        net_agent: &mut TlsStream<TcpStream>,
    ) -> Result<Agent, AgentCommunicationError> {
        trace!("Trying to authenticate agent...");

        let result = self.unsafe_authenticate(net_agent).await;

        if let Err(e) = result {
            trace!("Authentication failed: {e}");

            net_agent
                .shutdown()
                .await
                .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;

            return Err(e);
        }

        // self.agent_repo
        //     .set_agents_net(net_agent.clone())
        //     .await
        //     .map_err(|e| AgentCommunicationError::FailedToSetAgentNetConnection(e))?;

        return result;
    }

    async fn check_agent_api_key(
        &self,
        agent: &Agent,
        net_agent: &mut TlsStream<TcpStream>,
    ) -> Result<(), AgentCommunicationError> {
        trace!("Receiving agent's API key...");

        let mut response = Vec::new();
        net_agent
            .read_until(b'\n', &mut response)
            .await
            .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;

        let response_str = String::from_utf8(response)
            .map_err(|e| AgentCommunicationError::InvalidApiKeyUtf8(e))?;

        if response_str.trim() != agent.api_key {
            trace!("Agent API key mismatch."); // Not showing the api keys for obvious reasons

            net_agent
                .write(b"ER")
                .await
                .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;

            return Err(AgentCommunicationError::AuthRejected);
        }

        trace!("Agent API key is correct!");

        net_agent
            .write(b"OK")
            .await
            .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;

        return Ok(());
    }

    async fn process_reverse_api_key(
        &self,
        agent: &Agent,
        net_agent: &mut TlsStream<TcpStream>,
    ) -> Result<(), AgentCommunicationError> {
        trace!("Sending agent's reverse API key...");

        net_agent
            .write(agent.reverse_api_key.clone().as_bytes())
            .await
            .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;

        let mut ack = [0u8; 2];
        net_agent
            .read(&mut ack)
            .await
            .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;

        if &ack != b"OK" {
            trace!("Agent did not acknowledge reverse API key :(");

            return Err(AgentCommunicationError::ReverseApiKeyNotAcknowledged);
        }

        trace!("Agent acknowledged reverse API key :)");

        return Ok(());
    }

    async fn unsafe_authenticate(
        &self,
        net_agent: &mut TlsStream<TcpStream>,
    ) -> Result<Agent, AgentCommunicationError> {
        trace!("Receiving agent's ID...");

        let agent = self.get_db_agent(net_agent).await?;

        trace!("Sending auth ack...");

        self.first_ack(net_agent).await?;

        trace!("Checking agent's API key...");

        self.check_agent_api_key(&agent, net_agent).await?;

        trace!("Sending agent's reverse API key...");

        self.process_reverse_api_key(&agent, net_agent).await?;

        return Ok(agent);
    }

    async fn first_ack(
        &self,
        net_agent: &mut TlsStream<TcpStream>,
    ) -> Result<(), AgentCommunicationError> {
        net_agent
            .write(b"AUTH")
            .await
            .map_err(|e| AgentCommunicationError::NetError(e.to_string()))?;
        Ok(())
    }
}
