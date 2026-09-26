use yaml_rust2::Yaml;

use crate::errors::ConfigError;

pub type GeneratorArguments = Vec<Yaml>;

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub generator_id: String,
    pub args: Vec<Yaml>,
}

impl GeneratorConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let generator_id = yaml["name"]
            .as_str()
            .ok_or(ConfigError::RunGeneratorNameMissing)?;

        if yaml["args"].is_badvalue() || !yaml["args"].is_array() {
            return Err(ConfigError::RunGeneratorArgsNotArray(
                generator_id.to_string(),
            ));
        }
        let args_raw = yaml["args"].as_vec().unwrap_or(&Vec::new()).clone();

        return Ok(Self {
            generator_id: String::from(generator_id),
            args: args_raw,
        });
    }
}
