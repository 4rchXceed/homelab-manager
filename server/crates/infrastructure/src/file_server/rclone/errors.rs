use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileServerErrors {
    #[error("Reached timeout while waiting for the file server to start")]
    TimeoutError,
    #[error("The file server proxy is already started")]
    ProxyAlreadyStarted,
}
