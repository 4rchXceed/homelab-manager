use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum ToAgentMessage {
    CheckStorage(String, bool), // (path, can be created)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FromAgentMessage {
    StorageCheckResult(bool), // is valid
}

#[derive(Serialize, Deserialize)]
pub struct ToAgentMessageWrapper {
    pub message: ToAgentMessage,
    pub id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct FromAgentMessageWrapper {
    pub message: FromAgentMessage,
    pub id: u32,
}
