use log::error;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use application::generators::usecases::errors::GenerateConfigError;
use application::generators::usecases::generate::GenerateConfig;
use application::net::repositories::agent_connection::AgentConnection;
use application::{
    agent::repositories::agents_repository::AgentsRepository,
    database::repositories::services_db::ServicesDb,
    services::repositories::services_repository::ServicesRepository,
};
use async_trait::async_trait;
use config::service::config::ServiceConfig;
use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};
use domain::services::service::Service;
use thiserror::Error;
use tokio::sync::RwLock;
use turso::core::alloc::Vec;
use log::trace;

#[derive(Debug, Error)]
pub enum ServicesRepositoryError {
    #[error("Service with ID: {0} not found")]
    ServiceNotFound(String),
    #[error("Operation requires an agent on {0}, but no agent is assigned")]
    NoAgentAssigned(String),
    #[error("Operation requires a connected agent for {0}, but agent {1} isn't connected")]
    AgentNotConnected(String, String),
    #[error("Failed to generate config for {0} on agent {1}: {2}")]
    FailedToGenerate(String, String, GenerateConfigError),
    #[error("Failed to reload service {0} on agent {1}: {2}")]
    FailedToReloadService(String, String, String),
}

pub struct InMemoryServicesRepository {
    services: Arc<RwLock<Vec<Service>>>,
    agents: Arc<dyn AgentsRepository>,
    services_db: Arc<dyn ServicesDb>,
    generate: GenerateConfig,
}

impl InMemoryServicesRepository {
    pub fn new(
        agents: Arc<dyn AgentsRepository>,
        services_db: Arc<dyn ServicesDb>,
        generate_usecase: GenerateConfig,
    ) -> Self {
        return Self {
            services: Arc::new(RwLock::new(Vec::new())),
            agents: agents,
            services_db: services_db,
            generate: generate_usecase,
        };
    }

    pub async fn reload_service(
        &self,
        service_id: String,
        net_agent: Arc<dyn AgentConnection>,
        commands: Vec<String>,
        timeout: u64,
    ) -> Result<(), String> {
        trace!("Reloading service {service_id} with {} commands, and a timeout of {timeout} seconds", commands.len());

        let result = net_agent
            .send_pingpong(
                ServerToAgentMsg::RunCommands {
                    service: service_id,
                    commands: commands,
                },
                Duration::from_secs(timeout),
            )
            .await;

        match result {
            Ok(msg) => match msg {
                AgentToServerMsg::RunCommandsResponse {
                    success,
                    error_message,
                } => {
                    if success {
                        trace!("Service reloaded successfully!");
                        Ok(())
                    } else {
                        Err(error_message)
                    }
                }
                _ => Err(String::from("Unexpected response from agent")),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn regenerate(
        &self,
        service_id: String,
    ) -> Result<(), ServicesRepositoryError> {
        trace!("Regenerating service {service_id}");

        let services = self.services.read().await;

        let service_found = services
            .iter()
            .find(|s| s.id == service_id)
            .ok_or(ServicesRepositoryError::ServiceNotFound(service_id.clone()))?;

        let agent_id = service_found
            .agent_host_id
            .as_ref()
            .ok_or(ServicesRepositoryError::NoAgentAssigned(service_id.clone()))?;

        let net_agent = self.agents.get_agent_net(agent_id.clone()).await.ok_or(
            ServicesRepositoryError::AgentNotConnected(service_id.clone(), agent_id.clone()),
        )?;

        for config_file in &service_found.config_files {
            trace!("Regenerating config file: {}", config_file.path);

            for generator in &config_file.service_generators {
                trace!("Running generator {} for config file: {}", generator.base, config_file.path);

                let generate_result = self
                    .generate
                    .generate(config_file.path.clone(), generator, net_agent.clone())
                    .await;

                if let Err(e) = generate_result {
                    trace!("Failed to generate config file: {} with error: {}", config_file.path, e);

                    // Stop the entire operation, so the user is forced to fix the issue before continuing
                    return Err(ServicesRepositoryError::FailedToGenerate(
                        service_id.clone(),
                        agent_id.clone(),
                        e,
                    ));
                }
            }

            if config_file.service_generators.len() > 0
                && config_file.commands_when_updated.len() > 0
            {
                trace!("Service requires reload...");
                let reload_result = self
                    .reload_service(
                        service_id.clone(),
                        net_agent.clone(),
                        config_file.commands_when_updated.clone(),
                        config_file.reload_timeout,
                    )
                    .await;

                if let Err(error) = reload_result {
                    return Err(ServicesRepositoryError::FailedToReloadService(
                        service_id.clone(),
                        agent_id.clone(),
                        error,
                    ));
                }
            }
        }

        return Ok(());
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

    async fn assign_to(&self, service_id: String, agent_id: String) -> Result<(), String> {
        {
            trace!("Assigning service {service_id} to agent {agent_id}");
            let mut services_write = self.services.write().await;

            let service = services_write
                .iter_mut()
                .find(|s| s.id == service_id)
                .ok_or(format!("Service with ID: {} not found", service_id))?;

            self.services_db
                .update_agent_id_for(service.id.clone(), agent_id.clone())
                .await
                .map_err(|e| format!("Failed to ensure service into db: {}", e))?;

            service.agent_host_id = Some(agent_id.clone());
        }

        self.regenerate(service_id.clone())
            .await
            .map_err(|e| format!("Failed to regenerate service: {}", e))?;

        error!("TODO: Implement stop");

        return Ok(());
    }

    async fn reload(&self, _: String) -> Result<(), String> {
        // TODO
        return Ok(());
    }
}
