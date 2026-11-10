use std::path::Path;

use thiserror::Error;
use tokio::fs::copy;

use crate::context::AgentContext;

#[derive(Debug, Error)]
pub enum CopySampleError {
    #[error("Sample file not found: {0}")]
    SampleFileNotFound(String),
    #[error("Failed to copy sample file: {0}")]
    FailedToCopySampleFile(String),
}

pub async fn copy_sample_file(
    service: String,
    file_name: String,
    context: AgentContext,
) -> Result<(), CopySampleError> {
    let service_path = {
        let config = context.config.read().await;

        Path::new(&config.service_folder).join(&service)
    };

    let file_path = service_path.join(file_name);

    let sample_file_path = file_path.with_extension("sample");
    if !sample_file_path.exists() {
        return Err(CopySampleError::SampleFileNotFound(
            sample_file_path.display().to_string(),
        ));
    }
    copy(sample_file_path, file_path)
        .await
        .map_err(|e| CopySampleError::FailedToCopySampleFile(e.to_string()))?;

    return Ok(());
}
