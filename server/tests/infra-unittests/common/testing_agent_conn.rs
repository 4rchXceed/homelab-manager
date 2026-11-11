use std::{sync::Arc, time::Duration};

use application::net::repositories::agent_connection::{AgentConnection, MessageId};
use async_trait::async_trait;
use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};
use tokio::sync::RwLock;

pub struct TestingAgentConnection {
    // recv_list: Arc<RwLock<Vec<AgentToServerMsg>>>,
    sent_list: Arc<RwLock<Vec<ServerToAgentMsg>>>,
}

impl TestingAgentConnection {
    pub fn new() -> Self {
        Self {
            // recv_list: Arc::new(RwLock::new(Vec::new())),
            sent_list: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // pub async fn get_recv(&self) -> Vec<AgentToServerMsg> {
    //     let recv_list = self.recv_list.read().await;

    //     return recv_list.clone();
    // }

    pub async fn get_sent(&self) -> Vec<ServerToAgentMsg> {
        let recv_list = self.sent_list.read().await;

        return recv_list.clone();
    }
}

#[async_trait]
impl AgentConnection for TestingAgentConnection {
    async fn close(&self) -> Result<(), String> {
        Ok(())
    }
    async fn start_processing(&self) {}

    async fn send_message(&self, _message: ServerToAgentMsg) -> Result<MessageId, String> {
        Ok(String::from("testmessage"))
    }

    async fn receive_message(&self, _id: MessageId) -> Result<AgentToServerMsg, String> {
        Ok(AgentToServerMsg::Void)
    }

    async fn send_pingpong(
        &self,
        message: ServerToAgentMsg,
        _timeout: Duration,
    ) -> Result<AgentToServerMsg, String> {
        self.sent_list.write().await.push(message.clone());

        match message {
            ServerToAgentMsg::CopySampleFile {
                service: _,
                file: _,
            } => Ok(AgentToServerMsg::CopySampleResponse {
                success: true,
                error_message: None,
            }),
            ServerToAgentMsg::GenerateConfig {
                commands: _,
                generator_mode: _,
                service: _,
                image: _,
            } => Ok(AgentToServerMsg::GenerateConfigResponse {
                success: true,
                error_message: None,
            }),
            ServerToAgentMsg::RunCommands {
                service: _,
                commands: _,
            } => Ok(AgentToServerMsg::RunCommandsResponse {
                success: true,
                error_message: None,
            }),
            _ => panic!("Message: {message:?} not implemented!"),
        }
    }
}
