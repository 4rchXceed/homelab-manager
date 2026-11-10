use std::{env::var, path::Path};

use yaml_rust2::YamlLoader;

use crate::config::Config;
use crate::errors::{ConfigError, ConfigLoadError};
use consts::DEFAULT_CONFIG_FILE_PATH;

pub struct ConfigLoader {
    config_path: String,
}

impl ConfigLoader {
    pub fn new_from_env() -> Self {
        let config_path = var("CONFIG_FILE").unwrap_or(String::from(DEFAULT_CONFIG_FILE_PATH));
        return Self {
            config_path: config_path,
        };
    }

    pub fn with_path(config_path: String) -> Self {
        return Self {
            config_path: config_path,
        };
    }

    pub fn get_config_path(&self) -> Result<String, ()> {
        let path_internal = format!("{}", self.config_path); // .donottouch.internal
        if Path::new(path_internal.as_str()).exists() {
            return Ok(path_internal);
        } else {
            return Err(());
        }
    }

    pub fn load(&self) -> Result<(Config, String), ConfigLoadError> {
        let config_path = self
            .get_config_path()
            .map_err(|_| ConfigLoadError::FileNotFound)?;

        let config_content = std::fs::read_to_string(config_path)
            .map_err(|e| ConfigLoadError::ConfigReadError(e))?;

        let config_raw = YamlLoader::load_from_str(config_content.as_str())
            .map_err(|e| ConfigLoadError::YamlParseError(e))?;

        let res = self
            .post_process_config(config_raw)
            .map_err(|e| ConfigLoadError::ConfigError(e))?;

        return Ok((res, config_content));
    }

    pub fn post_process_config(
        &self,
        config_raw: Vec<yaml_rust2::Yaml>,
    ) -> Result<Config, ConfigError> {
        let first = config_raw.first().ok_or(ConfigError::NoConfig)?;

        let config = Config::from_yaml(first)?;

        return Ok(config);
    }
}
