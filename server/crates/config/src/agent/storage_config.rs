use yaml_rust2::Yaml;

use crate::errors::ConfigError;
use consts::DEFAULT_FALLBACK_STORAGE;

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub id: String,
    pub path: String,
    pub fallback: String,
    pub do_not_create: bool,
}

impl StorageConfig {
    pub fn from_yaml(yaml: &Yaml, id: String) -> Result<Self, ConfigError> {
        if yaml.as_str().is_some() {
            return Ok(Self {
                id: id,
                path: String::from(yaml.as_str().unwrap()),
                fallback: String::from(DEFAULT_FALLBACK_STORAGE),
                do_not_create: false,
            });
        }
        let path = yaml["path"]
            .as_str()
            .ok_or(ConfigError::AgentStoragePathMissing(id.clone()))?
            .to_string();

        let fallback = yaml["fallback"]
            .as_str()
            .unwrap_or(DEFAULT_FALLBACK_STORAGE)
            .to_string();

        let do_not_create = yaml["do_not_create"].as_bool().unwrap_or(false);

        return Ok(Self {
            id: id,
            path: path,
            fallback: fallback,
            do_not_create: do_not_create,
        });
    }
}
