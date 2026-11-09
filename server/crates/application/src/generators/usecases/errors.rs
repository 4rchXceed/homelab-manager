use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerateConfigError {
    #[error("Failed to get generator: {0}")]
    FailedToGetGenerator(String),
    #[error("Failed to generate commands: {0}")]
    FailedToGenerateCommands(String),
    #[error("Failed to send to agent: {0}")]
    FailedToSendToAgent(String),
    #[error("Unexpected response from agent")]
    UnexpectedResponse,
    #[error("Failed to generate config: {0}")]
    FailedToGenerateConfig(String),
    #[error("Failed to process variable for service {0}: {1}")]
    FailedToProcessUserVars(String, String),
    #[error("Variable is missing the 'type' attribute for service: {0}")]
    VariableMissingTypeAttribute(String),
}
