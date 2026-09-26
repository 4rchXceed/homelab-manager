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
}

#[derive(Serialize, Debug)]
pub enum ToServerMessageContent {
    Void,
}
