use std::collections::HashMap;

use async_trait::async_trait;
use config::service::config::ServiceConfig;

#[async_trait]
pub trait ServicesRepository {
    async fn ensure_services(&self, services: HashMap<String, ServiceConfig>)
    -> Result<(), String>;
    async fn assign_to(&self, agent_id: String) -> Result<(), String>;
    async fn reload(&self, service_id: String) -> Result<(), String>;
}
