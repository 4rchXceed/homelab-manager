use yaml_rust2::Yaml;

use crate::{
    config::{config::ConfigError, service::config_file::generator::config::GeneratorConfig},
    utils::fs::parse_time,
};

#[derive(Debug, Clone)]
pub struct ConfigFileConfig {
    pub path: String,
    pub when_config_updated: Vec<String>,
    pub reload_timeout: usize,
    pub generators_config: Vec<GeneratorConfig>,
}

impl ConfigFileConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let path = yaml["path"]
            .as_str()
            .ok_or(ConfigError::ConfigFileConfigPathMissing)?;
        let when_config_updated = yaml["when_config_updated"]
            .as_vec()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or(ConfigError::ConfigFileWhenConfigUpdatedCommandNotString(
                        path.to_string(),
                    ))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>, ConfigError>>()?;

        let reload_timeout_raw = yaml["reload_timeout"].as_str().unwrap_or("5s");
        let reload_timeout = parse_time(reload_timeout_raw);
        if reload_timeout.is_err() {
            return Err(ConfigError::ConfigFileReloadTimeoutParseError(
                path.to_string(),
                reload_timeout_raw.to_string(),
            ));
        }
        let reload_timeout = reload_timeout.unwrap();

        if yaml["generators"].is_badvalue() || !yaml["generators"].is_array() {
            return Err(ConfigError::ServiceConfigGeneratorsNotArray(
                path.to_string(),
            ));
        }
        let generators_config_raw = yaml["generators"].as_vec().unwrap();

        let generator_config = Vec::new();
        for generator_config_raw in generators_config_raw.iter() {
            let generator_config = GeneratorConfig::from_yaml(generator_config_raw);
            if generator_config.is_err() {
                return Err(generator_config.err().unwrap());
            }
        }

        return Ok(Self {
            path: String::from(path),
            when_config_updated: when_config_updated,
            reload_timeout: reload_timeout,
            generators_config: generator_config,
        });
    }
}
