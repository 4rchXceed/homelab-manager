use std::{sync::Arc, time::Duration};

use config::generator::config::GeneratorBaseConfig;
use domain::agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg};

use crate::{
    generators::{
        repositories::generator_registry::GeneratorRegistry, usecases::errors::GenerateConfigError,
    },
    net::repositories::agent_connection::AgentConnection,
};

pub struct GenerateConfig {
    pub agent: Arc<dyn AgentConnection>,
    pub generator_registry: Arc<dyn GeneratorRegistry>,
}

impl GenerateConfig {
    pub fn new(
        agent: Arc<dyn AgentConnection>,
        generator_registry: Arc<dyn GeneratorRegistry>,
    ) -> Self {
        return Self {
            agent,
            generator_registry,
        };
    }

    pub async fn generate(&self, config: &GeneratorBaseConfig) -> Result<(), GenerateConfigError> {
        let generator = self
            .generator_registry
            .generator_for_name(config.generator_base_name.clone())
            .ok_or(GenerateConfigError::GeneratorNotFound(
                config.generator_base_name.clone(),
            ))?;

        let commands = generator
            .generate_commands(&config.arguments)
            .map_err(|e| GenerateConfigError::FailedToGenerateCommands(e))?;

        let response = self
            .agent
            .send_pingpong(
                ServerToAgentMsg::GenerateConfig {
                    commands: commands,
                    requires_sample_file: config.require_sample_file,
                },
                Duration::from_secs(config.timeout as u64),
            )
            .await
            .map_err(|e| GenerateConfigError::GeneratorNotFound(e))?;

        match response {
            AgentToServerMsg::GenerateConfigResponse {
                success,
                error_message,
            } => {
                if !success {
                    return Err(GenerateConfigError::FailedToGenerateConfig(error_message));
                }
            }
            _ => {
                return Err(GenerateConfigError::UnexpectedResponse);
            }
        }

        return Ok(());
    }
}
