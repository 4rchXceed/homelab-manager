use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use config::service::config::ServiceConfig;

use crate::interactions::repositories::user_context::UserContext;

#[async_trait]
pub trait ServicesRepository {
    async fn ensure_services(&self, services: HashMap<String, ServiceConfig>)
    -> Result<(), String>;

    async fn assign_to(
        &self,
        service_id: String,
        agent_id: String,
        user_context: Arc<dyn UserContext>,
    ) -> Result<(), String>;

    async fn reload(&self, service_id: String) -> Result<(), String>;
}
