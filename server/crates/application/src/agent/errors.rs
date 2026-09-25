use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageInvalidity {
    #[error("Invalid UTF-8 message received: {0}")]
    InvalidUtf8(std::string::FromUtf8Error),
    #[error("Invalid message received: {0}")]
    InvalidJson(String),
}

#[derive(Error, Debug)]
pub enum AgentCommunicationError {
    #[error("Agent is not connected (maybe disconnected / not initialized)")]
    AgentNotConnected,
    #[error("Auth failed: {0}")]
    AuthFailed(String),
    #[error("Client tried to connect with wrong API key, connection rejected")]
    AuthRejected,
    #[error("Agent did not acknowledge reverse API key")]
    ReverseApiKeyNotAcknowledged,
    #[error("Invalid message received: {0}")]
    InvalidMessage(MessageInvalidity),
    #[error("Agent's api key isn't UTF-8")]
    InvalidApiKeyUtf8(std::string::FromUtf8Error),
    #[error("Agent's api key isn't UTF-8")]
    InvalidAgentIdUtf8(std::string::FromUtf8Error),
    #[error("Ping-Pong timed out")]
    PingPongTimeout,
    #[error("Net error: {0}")]
    NetError(String),
    #[error("Failed to get agent from database: {0}")]
    FailedToGetAgentFromDb(String),
    #[error("Agent with ID: {0} not found in database")]
    AgentWithIdNotFound(String),
    // #[error(
    //     "Failed to unlock context for writing (you should not see this error, please report it)"
    // )]
    // ContextUnlockFailed(String),
}
