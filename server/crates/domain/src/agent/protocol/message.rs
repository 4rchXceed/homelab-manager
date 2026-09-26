pub enum ServerToAgentMsg {
    GenerateConfig {
        commands: Vec<String>,
        requires_sample_file: bool,
    },
    Void,
}
pub enum AgentToServerMsg {
    GenerateConfigResponse {
        success: bool,
        error_message: String,
    },
    Void,
}
