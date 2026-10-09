pub enum GeneratorMode {
    Sandboxed,
    Unsandboxed,
}

pub enum ServerToAgentMsg {
    GenerateConfig {
        commands: Vec<String>,
        requires_sample_file: bool,
        file_path: String,
        generator_mode: GeneratorMode,
    },
    RunCommands {
        service: String,
        commands: Vec<String>,
    },
    Void,
}
pub enum AgentToServerMsg {
    GenerateConfigResponse {
        success: bool,
        error_message: String,
    },
    RunCommandsResponse {
        success: bool,
        error_message: String,
    },
    Void,
}
