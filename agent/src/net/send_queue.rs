use std::sync::Arc;

use tokio::sync::RwLock;

use crate::net::message::{ToServerMessage, ToServerMessageContent};

#[derive(Clone)]
pub struct SendQueue {
    pub queue: Arc<RwLock<Vec<ToServerMessage>>>,
    pub msg_id: Option<String>,
}

impl SendQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(Vec::new())),
            msg_id: None,
        }
    }

    pub async fn send(&self, message: ToServerMessageContent) {
        self.queue.write().await.push(ToServerMessage {
            id: self.msg_id.clone().unwrap_or_default(),
            content: message,
        });
    }
}
