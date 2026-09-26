use std::collections::HashMap;

use utils::config::parse_time;
use yaml_rust2::Yaml;

use crate::errors::ConfigError;

pub type Arguments = HashMap<String, Yaml>;

#[derive(Debug, Clone)]
pub struct GeneratorBaseConfig {
    pub id: String,
    pub generator_base_name: String,
    pub require_sample_file: bool,
    pub arguments: HashMap<String, Yaml>,
    pub timeout: usize,
}

impl GeneratorBaseConfig {
    pub fn from_yaml(yaml: &Yaml, key: String) -> Result<Self, ConfigError> {
        let base = yaml["base"]
            .as_str()
            .ok_or(ConfigError::GeneratorConfigBaseMissing(key.clone()))?;

        let arguments = yaml
            .as_hash()
            .ok_or(ConfigError::GeneratorConfigArgumentsMissing(key.clone()))?
            .iter()
            .map(|(k, v)| (k.as_str().unwrap_or_default().to_string(), v.clone()))
            .collect();

        let require_sample_file = yaml["requires_sample_file"].as_bool().unwrap_or(false);

        let timeout = parse_time(yaml["timeout"].as_str().unwrap_or("30s"))
            .map_err(|e| ConfigError::GeneratorConfigTimeoutParseError(key.clone(), e))?;

        return Ok(Self {
            id: key,
            generator_base_name: base.to_string(),
            arguments: arguments,
            require_sample_file: require_sample_file,
            timeout: timeout,
        });
    }
}
