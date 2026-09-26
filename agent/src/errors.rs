use log::SetLoggerError;
use thiserror::Error;

use crate::{
    db::database::{DatabaseOpenError, DatabaseSaveError},
    file_client::errors::RCloneCommandError,
};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("The keyword 'AUTH' was not sent by the server")]
    AuthAckNotSent,
    #[error("The API key has been rejected by the server")]
    InvalidApiKey,
    #[error("Could not verify server's identity: reverse api key invalid")]
    InvalidReverseApiKey,
    #[error("The reverse api key isn't utf8 (aka a string)")]
    ReverseApiKeyIsntUtf8,
    #[error("Could not save the database after inserting the reverse api key")]
    NewReverseApiKeyDbSave(DatabaseSaveError),
}

#[derive(Debug, Error)]
pub enum NetInitError {
    #[error("Error while creating the TLS connection: {0}")]
    TlsCreationError(#[from] crate::net::error::NetError),
    #[error("Invalid server host")]
    InvalidServerHost,
    #[error("Server connection error (at address: {1}): {0}")]
    ServerConnectionError(std::io::Error, String),
    #[error("TLS error: {0}")]
    TlsError(String),
}

#[derive(Debug, Error)]
pub enum NetRunError {
    #[error("Socket close error: {0}")]
    SocketCloseError(std::io::Error),
    #[error("The socket is closed / wasn't created")]
    SocketClosedOrNotFound,
    #[error("Socket error: {0}")]
    SocketError(tokio::io::Error),
    #[error("Socket write: {0}")]
    SocketWriteError(std::io::Error),
}

#[derive(Debug, Error)]
pub enum ClientRuntimeError {
    #[error("Authentication with server error")]
    AuthError(AuthError),
    #[error("Network error while running: {0}")]
    NetRuntimeError(NetRunError),
    #[error("Network initialisation error: {0}")]
    NetInitError(NetInitError),
    #[error("File sync error (rclone): {0}")]
    FileSyncError(RCloneCommandError),
}

impl ClientRuntimeError {
    pub fn auth(error: AuthError) -> Self {
        return Self::AuthError(error);
    }

    pub fn net_run(error: NetRunError) -> Self {
        return Self::NetRuntimeError(error);
    }

    pub fn net_init(error: NetInitError) -> Self {
        return Self::NetInitError(error);
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to open ini file: {0}")]
    FailedToOpenIniFile(String),
    #[error("Missing required section: {0} in ini file")]
    MissingSection(String),
    #[error("Missing required key: {0} in section: {1} of ini file")]
    MissingKey(String, String),
}

#[derive(Debug, Error)]
pub enum RequirementMissing {
    #[error("RClone is not installed or not found in PATH")]
    RCloneNotFound,
}

#[derive(Debug, Error)]
pub enum ClientInitError {
    #[error("Configuration error: {0}")]
    ConfigError(ConfigError),
    #[error("Failed to open database: {0} (path: {1}")]
    OpenDbError(DatabaseOpenError, String),
    #[error("Failed to create logger (simple_logger): {0}")]
    LogInitError(SetLoggerError),
    #[error("Invalid log level")]
    LogLevelError,
    #[error("Requirement missing: {0}")]
    RequirementMissing(RequirementMissing),
}
