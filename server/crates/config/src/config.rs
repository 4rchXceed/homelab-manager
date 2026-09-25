use std::collections::HashMap;

use yaml_rust2::Yaml;

use crate::errors::ConfigError;
use crate::{
    agent::config::AgentConfig, generator::config::GeneratorConfig, generic::config::GeneralConfig,
    service::config::ServiceConfig,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub config_general: GeneralConfig,
    pub generators_config: HashMap<String, GeneratorConfig>,
    pub services_config: HashMap<String, ServiceConfig>,
    pub agents_config: Vec<AgentConfig>,
}

impl Config {
    pub fn from_yaml(yaml: &Yaml) -> Result<Config, ConfigError> {
        if yaml["config"].is_badvalue() {
            return Err(ConfigError::NoGeneralConfig);
        }
        let config_general = GeneralConfig::from_yaml(&yaml["config"]);
        if config_general.is_err() {
            return Err(config_general.err().unwrap());
        }
        let config_general = config_general.unwrap();

        let mut generators_config: HashMap<String, GeneratorConfig> = HashMap::new();
        if yaml["generators"].is_badvalue() {
            return Err(ConfigError::NoGeneratorsConfig);
        }
        if !yaml["generators"].is_hash() {
            return Err(ConfigError::GeneratorsConfigNotObject);
        }
        let generators_config_raw = yaml["generators"].as_hash().unwrap();
        for (key, value) in generators_config_raw.iter() {
            let key_str = key.as_str();
            if key_str.is_none() {
                return Err(ConfigError::OneGeneratorKeyNotFound);
            }
            let key_str = key_str.unwrap();

            let generator_config = GeneratorConfig::from_yaml(value, String::from(key_str));
            if generator_config.is_err() {
                return Err(generator_config.err().unwrap());
            }
            generators_config.insert(key_str.to_string(), generator_config.unwrap());
        }

        if yaml["services"].is_badvalue() || !yaml["services"].is_hash() {
            return Err(ConfigError::ServicesConfigNotObject);
        }
        let services_config_raw = yaml["services"].as_hash().unwrap();

        let mut services_config: HashMap<String, ServiceConfig> = HashMap::new();
        for (key, value) in services_config_raw.iter() {
            let key_str = key.as_str();
            if key_str.is_none() {
                return Err(ConfigError::OneServiceKeyNotFound);
            }
            let key_str = key_str.unwrap();
            let service_config = ServiceConfig::from_yaml(value, String::from(key_str));
            if service_config.is_err() {
                return Err(service_config.err().unwrap());
            }
            services_config.insert(key_str.to_string(), service_config.unwrap());
        }

        let agents_config_raw = yaml["agents"]
            .as_vec()
            .ok_or(ConfigError::AgentConfigMissing)?;

        let mut agents_config: Vec<AgentConfig> = Vec::new();
        for agent_config_raw in agents_config_raw.iter() {
            let agent_config = AgentConfig::from_yaml(agent_config_raw)?;
            agents_config.push(agent_config);
        }

        return Ok(Self {
            config_general: config_general,
            generators_config: generators_config,
            services_config: services_config,
            agents_config: agents_config,
        });
    }
}
