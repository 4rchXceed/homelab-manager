use std::time::Duration;

use async_trait::async_trait;
use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};

pub type MessageId = String;

#[async_trait]
pub trait AgentConnection: Send + Sync {
    async fn close(&self) -> Result<(), String>;
    async fn start_processing(&self);
    // async fn stop_processing(&self) -> Result<(), String>;: TODO
    async fn send_message(&self, message: ServerToAgentMsg) -> Result<MessageId, String>;
    async fn receive_message(&self, id: MessageId) -> Result<AgentToServerMsg, String>;
    async fn send_pingpong(
        &self,
        message: ServerToAgentMsg,
        timeout: Duration,
    ) -> Result<AgentToServerMsg, String>;
}
