use thiserror::Error;

#[derive(Debug, Error)]
pub enum RCloneCommandError {
    #[error("Failed to execute rclone command: {0}")]
    CommandExecutionError(String),
    #[error("Failed to create directory: {0}")]
    DirectoryCreationError(String),
}
