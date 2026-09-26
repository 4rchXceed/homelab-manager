use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageInvalidity {
    #[error("Invalid UTF-8 message received: {0}")]
    InvalidUtf8(std::string::FromUtf8Error),
    #[error("Invalid message received: {0}")]
    InvalidJson(String),
}
