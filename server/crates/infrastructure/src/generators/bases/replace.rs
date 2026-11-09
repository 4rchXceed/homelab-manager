use application::generators::repositories::generator_base::GeneratorBase;
use async_trait::async_trait;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use config::generator::config::GeneratorBaseConfig;
use domain::agent::protocol::message::GeneratorMode;

pub struct ReplaceFileGeneratorBase {}

impl ReplaceFileGeneratorBase {
    pub fn new() -> Self {
        return Self {};
    }
}

#[async_trait]
impl GeneratorBase for ReplaceFileGeneratorBase {
    fn get_name(&self) -> &str {
        return "replace";
    }

    async fn generate_commands(
        &self,
        config: &GeneratorBaseConfig,
        arguments: &Vec<String>,
        file_path: &String,
    ) -> Result<(Vec<String>, GeneratorMode, Option<String>), String> {
        if let Some(replace_from) = &config.arguments.get("string")
            && let Some(replace_from_str) = replace_from.as_str()
        {
            if let Some(replace_to) = &arguments.get(0) {
                let default_image = String::from("python:3.12.15-slim");

                let image = if let Some(image) = &config.arguments.get("base_image") {
                    Some(image.as_str().unwrap_or(&default_image).to_string())
                } else {
                    Some(default_image)
                };

                let replace_code = format!(
                    include_str!("replace_script.py"),
                    BASE64_STANDARD.encode(replace_from_str),
                    BASE64_STANDARD.encode(replace_to),
                    file_path,
                    file_path
                );

                let commands = vec![
                    format!("cat <<EOF > /tmp/replace_script.py\n{}\nEOF", replace_code),
                    String::from("python3 /tmp/replace_script.py"),
                ];

                return Ok((commands, GeneratorMode::Sandboxed, image));
            } else {
                return Err(String::from("Missing required service argument"));
            }
        } else {
            return Err(String::from(
                "Missing required argument: 'string' key => string to replace",
            ));
        }
    }
}
