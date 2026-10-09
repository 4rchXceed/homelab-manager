use std::{sync::Arc, time::Duration};

use domain::{
    agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg},
    services::generator::service_generator::ServiceGenerator,
};

use crate::{
    generators::{
        repositories::generator_bases_registry::GeneratorBaseRegistry,
        usecases::errors::GenerateConfigError,
    },
    net::repositories::agent_connection::AgentConnection,
};

pub struct GenerateConfig {
    pub generator_registry: Arc<dyn GeneratorBaseRegistry>,
}

impl GenerateConfig {
    pub fn new(generator_registry: Arc<dyn GeneratorBaseRegistry>) -> Self {
        return Self { generator_registry };
    }

    pub async fn generate(
        &self,
        path: String,
        config: &ServiceGenerator,
        agent: Arc<dyn AgentConnection>,
    ) -> Result<(), GenerateConfigError> {
        let (generator, generator_base_cfg) = self
            .generator_registry
            .generator_for_name(config.base.clone())
            .await.map_err(|e|GenerateConfigError::FailedToGetGenerator(e))?;

        let (commands, mode) = generator
            .generate_commands(&config.arguments)
            .map_err(|e| GenerateConfigError::FailedToGenerateCommands(e))?;

        let response = agent
            .send_pingpong(
                ServerToAgentMsg::GenerateConfig {
                    commands: commands,
                    requires_sample_file: generator_base_cfg.require_sample_file,
                    file_path: path,
                    generator_mode: mode,
                },
                Duration::from_secs(generator_base_cfg.timeout as u64),
            )
            .await
            .map_err(|e| GenerateConfigError::FailedToSendToAgent(e))?;

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
