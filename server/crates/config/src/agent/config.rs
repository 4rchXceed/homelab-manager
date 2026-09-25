use std::{collections::HashMap, net::IpAddr};

use yaml_rust2::{Yaml, yaml::Hash};

use crate::{agent::storage_config::StorageConfig, errors::ConfigError};

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub id: String,
    pub description: String,
    pub ip: IpAddr,
    pub api_key: String,
    pub storages: HashMap<String, StorageConfig>,
}

impl AgentConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let id = yaml["id"]
            .as_str()
            .ok_or(ConfigError::AgentIdMissing)?
            .to_string();

        let description = yaml["description"]
            .as_str()
            .unwrap_or("No description provided")
            .to_string();

        let ip_str = yaml["ip"]
            .as_str()
            .ok_or(ConfigError::AgentIpMissing(id.clone()))?;
        let ip: IpAddr = ip_str
            .parse()
            .map_err(|_| ConfigError::AgentIpInvalid(id.clone(), ip_str.to_string()))?;

        let api_key = yaml["api_key"]
            .as_str()
            .ok_or(ConfigError::AgentApiKeyMissing(id.clone()))?
            .to_string();

        let empty_hash = Hash::new();
        let storages_raw = yaml["storages"].as_hash().unwrap_or(&empty_hash);
        let mut storages = HashMap::new();

        for (key, storage) in storages_raw.iter() {
            let storage_id = key
                .as_str()
                .ok_or(ConfigError::AgentStorageIdInvalid(id.clone()))?;
            let storage_config = StorageConfig::from_yaml(storage, storage_id.to_string())?;
            storages.insert(storage_id.to_string(), storage_config);
        }

        return Ok(Self {
            id: id,
            description: description,
            ip: ip,
            api_key: api_key,
            storages: storages,
        });
    }
}
