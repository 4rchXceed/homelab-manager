use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageInvalidity {
    #[error("Invalid UTF-8 message received: {0}")]
    InvalidUtf8(std::string::FromUtf8Error),
    #[error("Invalid JSON message received: {0}")]
    InvalidJson(serde_json::Error),
}

#[derive(Error, Debug)]
pub enum AgentCommunicationError {
    #[error("Agent is not connected (maybe disconnected / not initialized)")]
    AgentNotConnected,
    #[error("Failed to send message to agent: {0}")]
    FailedToSendMessage(std::io::Error),
    #[error("Failed to receive message from agent: {0}")]
    FailedToReceiveMessage(std::io::Error),
    #[error("Auth failed: {0}")]
    AuthFailed(String),
    #[error("Client tried to connect with wrong API key, connection rejected")]
    AuthRejected,
    #[error("Agent did not acknowledge reverse API key")]
    ReverseApiKeyNotAcknowledged,
    #[error("Failed to unlock agent for writing (you should not see this error, please report it)")]
    AgentUnlockFailed(String),
    #[error("Failed to read from socket: {0}")]
    ReadError(std::io::Error),
    #[error("Failed to write to socket: {0}")]
    WriteError(std::io::Error),
    #[error("Invalid message received: {0}")]
    InvalidMessage(MessageInvalidity),
    #[error("Ping-Pong timed out")]
    PingPongTimeout,
    // #[error(
    //     "Failed to unlock context for writing (you should not see this error, please report it)"
    // )]
    // ContextUnlockFailed(String),
}
