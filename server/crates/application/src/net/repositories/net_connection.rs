use async_trait::async_trait;

#[async_trait]
pub trait NetworkConnection: Send + Sync {
    async fn send_raw(&self, data: Vec<u8>) -> Result<(), String>;

    async fn recv_raw(&self, length: usize) -> Result<Vec<u8>, String>;

    async fn recv_until(&self, delimiter: u8) -> Result<Vec<u8>, String>;

    async fn close(&self) -> Result<(), String>;
}
