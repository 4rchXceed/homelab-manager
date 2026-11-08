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
        service: String,
        image: Option<String>,
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
        error_message: Option<String>,
    },
    RunCommandsResponse {
        success: bool,
        error_message: Option<String>,
    },
    Void,
}
