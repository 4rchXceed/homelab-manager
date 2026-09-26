use std::sync::Arc;

use async_trait::async_trait;

use crate::net::repositories::agent_connection::AgentConnection;

#[async_trait]
pub trait NetworkManager: Send + Sync {
    async fn accept_new_auth_agent(&self) -> Result<Arc<dyn AgentConnection>, String>;
}
