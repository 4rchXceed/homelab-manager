use std::sync::Arc;

use async_trait::async_trait;

use crate::net::repositories::net_connection::NetworkConnection;

#[async_trait]
pub trait NetworkManager: Send + Sync {
    async fn accept_new(&self) -> Result<Arc<dyn NetworkConnection>, String>;
}
