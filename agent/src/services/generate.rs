use std::path::Path;

use thiserror::Error;
use tokio::fs::copy;

use crate::context::AgentContext;

use super::run_cmd::{CommandError, run_commands};

#[derive(Debug, Error)]
pub enum ConfigGenerationError {
    #[error("Service folder: {0} not found. Please check the file server sync errors.")]
    ServiceNotFound(String),
    #[error("Sample file not found: {0}")]
    SampleFileNotFound(String),
    #[error("Failed to copy sample file: {0}")]
    FailedToCopySampleFile(String),
    #[error("Failed to generate config file: {0}")]
    FailedToGenerateConfigFile(CommandError),
}

pub async fn generate_config_file(
    mut commands: Vec<String>,
    requires_sample_file: bool,
    file_path: String,
    sandboxed: bool,
    service: String,
    context: AgentContext,
    image: Option<String>,
) -> Result<(), ConfigGenerationError> {
    let service_path = {
        let config = context.config.read().await;

        Path::new(&config.service_folder).join(&service)
    };

    let file_path = service_path.join(&file_path);

    if !service_path.exists() {
        return Err(ConfigGenerationError::ServiceNotFound(
            service_path.display().to_string(),
        ));
    }

    // TODO: Place this in a request, so the normal file doesnt get rewritten every time
    //     if requires_sample_file {
    //         let sample_file_path = file_path.with_extension("sample");
    //
    //         if !sample_file_path.exists() {
    //             return Err(ConfigGenerationError::SampleFileNotFound(
    //                 sample_file_path.display().to_string(),
    //             ));
    //         }
    //
    //         copy(sample_file_path, file_path)
    //             .await
    //             .map_err(|e| ConfigGenerationError::FailedToCopySampleFile(e.to_string()))?;
    //     }

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
