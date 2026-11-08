use std::{sync::Arc, time::Duration};

use domain::{
    agent::protocol::message::{AgentToServerMsg, ServerToAgentMsg},
    services::generator::service_generator::ServiceGenerator,
};
use log::error;

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
        service_name: String,
    ) -> Result<(), GenerateConfigError> {
        let (generator, generator_base_cfg) = self
            .generator_registry
            .generator_for_name(config.base.clone())
            .await
            .map_err(|e| GenerateConfigError::FailedToGetGenerator(e))?;

        error!("TODO: Implement arguments for generator base config");

        let (commands, mode, image) = generator
            .generate_commands(&generator_base_cfg, &Vec::new(), &path)
            .await
            .map_err(|e| GenerateConfigError::FailedToGenerateCommands(e))?;

        let response = agent
            .send_pingpong(
                ServerToAgentMsg::GenerateConfig {
                    commands: commands,
                    requires_sample_file: generator_base_cfg.require_sample_file,
                    file_path: path,
                    generator_mode: mode,
                    service: service_name,
                    image: image,
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
                    return Err(GenerateConfigError::FailedToGenerateConfig(
                        error_message.unwrap_or(String::from("Unknown error")),
                    ));
                }
            }
            _ => {
                return Err(GenerateConfigError::UnexpectedResponse);
            }
        }

        return Ok(());
    }
}
