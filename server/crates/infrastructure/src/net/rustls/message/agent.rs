use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};
use serde::{Deserialize, Serialize};

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
        };
    }
}

#[derive(Serialize)]
pub enum ToAgentMessageContent {
    Void,
}

#[derive(Deserialize)]
pub enum FromAgentMessageContent {
    Void,
}
