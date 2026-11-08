use async_trait::async_trait;

#[async_trait]
pub trait UserContext: Send + Sync {
    async fn print(&self, message: &str);

    async fn error_print(&self, message: &str);

    async fn prompt(&self, message: &str) -> Result<String, String>;
}
