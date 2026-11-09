use application::generators::repositories::generator_base::GeneratorBase;
use async_trait::async_trait;
use base64::prelude::*;
use config::generator::config::GeneratorBaseConfig;
use domain::agent::protocol::message::GeneratorMode;

pub struct BashGeneratorBase {}

impl BashGeneratorBase {
    pub fn new() -> Self {
        BashGeneratorBase {}
    }
}

#[async_trait]
impl GeneratorBase for BashGeneratorBase {
    fn get_name(&self) -> &str {
        return "bash";
    }

    async fn generate_commands(
        &self,
        config: &GeneratorBaseConfig,
        arguments: &Vec<String>,
        _file_path: &String,
    ) -> Result<(Vec<String>, GeneratorMode, Option<String>), String> {
        if let Some(commands) = &config.arguments.get("commands") {
            if let Some(commands_array) = commands.as_vec() {
                let mut commands: Vec<String> = Vec::new();

                commands.push(String::from("export DEBIAN_FRONTEND=noninteractive"));

                for (i, arg) in arguments.iter().enumerate() {
                    commands.push(format!(
                        "ARG{}=\"$(echo {} | base64 --decode)\"",
                        i,
                        BASE64_STANDARD.encode(arg)
                    ));
                }

                if let Some(deps_cmds) = &config.arguments.get("install_dependencies")
                    && let Some(deps_array) = deps_cmds.as_vec()
                {
                    let deps_str: Vec<String> = deps_array
                        .iter()
                        .filter_map(|cmd| cmd.as_str().map(|s| s.to_string()))
                        .collect();

                    commands.extend(deps_str);
                }

                let commands_str: Vec<String> = commands_array
                    .iter()
                    .filter_map(|cmd| cmd.as_str().map(|s| s.to_string()))
                    .collect();

                commands.extend(commands_str.clone());

                let image = config
                    .arguments
                    .get("base_image")
                    .and_then(|img| img.as_str().map(|s| s.to_string()));

                return Ok((commands, GeneratorMode::Sandboxed, image));
            } else {
                return Err(String::from(
                    "The commands argument MUST be an array of strings.",
                ));
            }
        } else {
            return Err(String::from(
                "The commands argument is required for bash generator.",
            ));
        }
    }
}
