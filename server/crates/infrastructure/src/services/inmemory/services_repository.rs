use std::{collections::HashMap, sync::Arc};

use application::{
    agent::repositories::agents_repository::AgentsRepository,
    database::repositories::services_db::ServicesDb,
    services::repositories::services_repository::ServicesRepository,
};
use async_trait::async_trait;
use config::service::config::ServiceConfig;
use domain::services::service::Service;
use tokio::sync::RwLock;

pub struct InMemoryServicesRepository {
    services: Arc<RwLock<Vec<Service>>>,
    agents: Arc<dyn AgentsRepository>,
    services_db: Arc<dyn ServicesDb>,
}

impl InMemoryServicesRepository {
    pub fn new(agents: Arc<dyn AgentsRepository>, services_db: Arc<dyn ServicesDb>) -> Self {
        return Self {
            services: Arc::new(RwLock::new(Vec::new())),
            agents: agents,
            services_db: services_db,
        };
    }
}

#[async_trait]
impl ServicesRepository for InMemoryServicesRepository {
    async fn ensure_services(
        &self,
        services: HashMap<String, ServiceConfig>,
    ) -> Result<(), String> {
        let mut services_write = self.services.write().await;

        for (_, service) in services {
            let service = self
                .services_db
                .ensure_service(service)
                .await
                .map_err(|e| format!("Failed to ensure service into db: {}", e))?;

            services_write.push(service);
        }

        return Ok(());
    }

    async fn assign_to(&self, _: String) -> Result<(), String> {
        // TODO
        return Ok(());
    }

    async fn reload(&self, _: String) -> Result<(), String> {
        // TODO
        return Ok(());
    }
}
