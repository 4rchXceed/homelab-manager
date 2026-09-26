use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};
use serde::{Deserialize, Serialize};
use turso::core::alloc::Vec;

#[derive(Serialize)]
pub struct ToAgentMessage {
    pub id: String,
    pub content: ToAgentMessageContent,
}

impl ToAgentMessage {
    pub fn from_domain_msg(id: String, content: ServerToAgentMsg) -> Self {
        return Self {
            id: id,
            content: match content {
                ServerToAgentMsg::GenerateConfig {
                    commands,
                    requires_sample_file,
                } => ToAgentMessageContent::GenerateConfig(commands, requires_sample_file),
                ServerToAgentMsg::Void => ToAgentMessageContent::Void,
            },
        };
    }
}

#[derive(Deserialize)]
pub struct FromAgentMessage {
    pub id: String,
    pub content: FromAgentMessageContent,
}

impl FromAgentMessage {
    pub fn to_domain_msg(&self) -> AgentToServerMsg {
        return match &self.content {
            FromAgentMessageContent::Void => AgentToServerMsg::Void,
            FromAgentMessageContent::GenerateConfigResponse(success, error_message) => {
                AgentToServerMsg::GenerateConfigResponse {
                    success: *success,
                    error_message: error_message.clone(),
                }
            }
        };
    }
}

#[derive(Serialize)]
pub enum ToAgentMessageContent {
    GenerateConfig(Vec<String>, bool),
    Void,
}

#[derive(Deserialize)]
pub enum FromAgentMessageContent {
    GenerateConfigResponse(bool, String),
    Void,
}
