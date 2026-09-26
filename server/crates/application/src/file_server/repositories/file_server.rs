use std::time::Duration;

use async_trait::async_trait;

#[async_trait]
pub trait FileServerProvider: Send + Sync {
    async fn start_threaded(&self) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn wait_until_started(&self, timeout: Duration) -> Result<(), String>;
}
