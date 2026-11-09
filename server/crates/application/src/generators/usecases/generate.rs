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
    interactions::repositories::user_context::UserContext,
    net::repositories::agent_connection::AgentConnection,
    var_provider::{
        context::VarProviderContext, repositories::var_provider_registry::VarProviderRegistry,
    },
};

pub struct GenerateConfig {
    pub generator_registry: Arc<dyn GeneratorBaseRegistry>,
    pub var_provider_registry: Arc<dyn VarProviderRegistry>,
}

impl GenerateConfig {
    pub fn new(
        generator_registry: Arc<dyn GeneratorBaseRegistry>,
        var_provider_registry: Arc<dyn VarProviderRegistry>,
    ) -> Self {
        return Self {
            generator_registry,
            var_provider_registry,
        };
    }

    pub async fn generate(
        &self,
        path: String,
        config: &ServiceGenerator,
        agent: Arc<dyn AgentConnection>,
        service_name: String,
        user_context: Arc<dyn UserContext>,
    ) -> Result<(), GenerateConfigError> {
        let (generator, generator_base_cfg) = self
            .generator_registry
            .generator_for_name(config.base.clone())
            .await
            .map_err(|e| GenerateConfigError::FailedToGetGenerator(e))?;

        let vars = self
            .process_variables(config, &service_name, user_context)
            .await?;

        let (commands, mode, image) = generator
            .generate_commands(&generator_base_cfg, &vars, &path)
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

    async fn process_variables(
        &self,
        config: &ServiceGenerator,
        service_name: &String,
        user_context: Arc<dyn UserContext>,
    ) -> Result<Vec<String>, GenerateConfigError> {
        let mut vars = Vec::new();

        for var in &config.arguments {
            if let Some(var_type) = var["type"].as_str() {
                let var_provider = self
                    .var_provider_registry
                    .var_provider_for_type(var_type.to_string())
                    .await
                    .map_err(|e| {
                        GenerateConfigError::FailedToProcessUserVars(
                            service_name.clone(),
                            e.to_string(),
                        )
                    })?;

                let real_value = var_provider
                    .process(
                        var.clone(),
                        service_name.clone(),
                        VarProviderContext {
                            user_context: user_context.clone(),
                        },
                    )
                    .await
                    .map_err(|e| {
                        GenerateConfigError::FailedToProcessUserVars(
                            service_name.clone(),
                            e.to_string(),
                        )
                    })?;

                vars.push(real_value);
            } else {
                return Err(GenerateConfigError::VariableMissingTypeAttribute(
                    service_name.clone(),
                ));
            }
        }

        return Ok(vars);
    }
}
