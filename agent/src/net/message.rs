use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct FromServerMessage {
    pub id: String,
    pub content: FromServerMessageContent,
}

#[derive(Serialize)]
pub struct ToServerMessage {
    pub id: String,
    pub content: ToServerMessageContent,
}

#[derive(Deserialize, Debug)]
pub enum FromServerMessageContent {
    Void,
    GenerateConfig(Vec<String>, bool, String, Option<String>),
    RunCommands(Vec<String>, String),
    CopySample(String, String),
}

#[derive(Serialize, Debug)]
pub enum ToServerMessageContent {
    Void,
    GenerateConfigResponse(bool, Option<String>),
    RunCommandsResponse(bool, Option<String>),
    CopySampleResponse(bool, Option<String>),
}
