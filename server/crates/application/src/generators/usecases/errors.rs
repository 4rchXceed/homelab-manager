use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerateConfigError {
    #[error("Generator not found: {0}")]
    GeneratorNotFound(String),
    #[error("Failed to generate commands: {0}")]
    FailedToGenerateCommands(String),
    #[error("Failed to send to agent: {0}")]
    FailedToSendToAgent(String),
    #[error("Unexpected response from agent")]
    UnexpectedResponse,
    #[error("Failed to generate config: {0}")]
    FailedToGenerateConfig(String),
}
