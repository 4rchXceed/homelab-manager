use std::path::Path;

use thiserror::Error;

use crate::context::AgentContext;

use super::run_cmd::{CommandError, run_commands};

#[derive(Debug, Error)]
pub enum ConfigGenerationError {
    #[error("Service folder: {0} not found. Please check the file server sync errors.")]
    ServiceNotFound(String),
    #[error("Failed to generate config file: {0}")]
    FailedToGenerateConfigFile(CommandError),
}

pub async fn generate_config_file(
    mut commands: Vec<String>,
    sandboxed: bool,
    service: String,
    image: Option<String>,
    context: AgentContext,
) -> Result<(), ConfigGenerationError> {
    let service_path = {
        let config = context.config.read().await;

        Path::new(&config.service_folder).join(&service)
    };

    if !service_path.exists() {
        return Err(ConfigGenerationError::ServiceNotFound(
            service_path.display().to_string(),
        ));
    }

    commands.insert(
        0,
        format!(
            "cd {}",
            match sandboxed {
                true => String::from("/service"),
                false => service_path.display().to_string(),
            }
        ),
    );

    run_commands(commands, service, sandboxed, context, image)
        .await
        .map_err(|e| ConfigGenerationError::FailedToGenerateConfigFile(e))?;

    return Ok(());
}
