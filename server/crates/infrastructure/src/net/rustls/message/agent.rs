use domain::agent::protocol::message::{AgentToServerMsg, GeneratorMode, ServerToAgentMsg};
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
                    file_path,
                    generator_mode,
                } => ToAgentMessageContent::GenerateConfig(
                    commands,
                    requires_sample_file,
                    file_path,
                    match generator_mode {
                        GeneratorMode::Sandboxed => true,
                        GeneratorMode::Unsandboxed => false,
                    },
                ),
                ServerToAgentMsg::RunCommands { service, commands } => {
                    ToAgentMessageContent::RunCommands(commands, service)
                }
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
            FromAgentMessageContent::RunCommandsResponse(success, error_message) => {
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
    GenerateConfig(Vec<String>, bool, String, bool),
    RunCommands(Vec<String>, String),
    Void,
}

#[derive(Deserialize)]
pub enum FromAgentMessageContent {
    GenerateConfigResponse(bool, String),
    RunCommandsResponse(bool, String),
    Void,
}
