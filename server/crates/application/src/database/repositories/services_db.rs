use async_trait::async_trait;
use config::service::config::ServiceConfig;
use domain::services::service::Service;

#[async_trait]
pub trait ServicesDb: Send + Sync {
    async fn ensure_service(&self, service: ServiceConfig) -> Result<Service, String>;
}
