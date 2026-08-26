use std::{env::var, path::Path};

use yaml_rust2::YamlLoader;

use crate::{
    config::config::{Config, ConfigError},
    consts::DEFAULT_CONFIG_FILE_PATH,
};

#[derive(Debug)]
pub enum ConfigLoadError {
    FileNotFound,
    ConfigReadError(std::io::Error),
    YamlParseError(yaml_rust2::ScanError),
    ConfigError(ConfigError),
}

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

    pub fn load(&mut self) -> Result<Config, ConfigLoadError> {
        let config_path = self.get_config_path();
        if config_path.is_err() {
            return Err(ConfigLoadError::FileNotFound);
        }
        let config_path = config_path.unwrap();

        let config_content = std::fs::read_to_string(config_path);
        if config_content.is_err() {
            return Err(ConfigLoadError::ConfigReadError(
                config_content.err().unwrap(),
            ));
        }
        let config_content = config_content.unwrap();

        let config_raw = YamlLoader::load_from_str(config_content.as_str());
        if config_raw.is_err() {
            return Err(ConfigLoadError::YamlParseError(config_raw.err().unwrap()));
        }
        let config_raw = config_raw.unwrap();

        let res = self.post_process_config(config_raw);
        if res.is_err() {
            return Err(ConfigLoadError::ConfigError(res.err().unwrap()));
        }
        return Ok(res.unwrap());
    }

    pub fn post_process_config(
        &mut self,
        config_raw: Vec<yaml_rust2::Yaml>,
    ) -> Result<Config, ConfigError> {
        let first = config_raw.first();
        if first.is_none() {
            return Err(ConfigError::NoConfig);
        }
        let first = first.unwrap();
        let config = Config::from_yaml(first);
        if config.is_err() {
            return Err(config.err().unwrap());
        }
        return Ok(config.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::fs::create_temp_dir;

    use super::*;

    static CONFIG_SAMPLE: &str = include_str!("../test_only/config_test.yaml");

    #[test]
    fn test_load_config() {
        let tmp_res = create_temp_dir();
        assert!(tmp_res.is_ok());
        let (tmp_path, tmp_folder) = tmp_res.unwrap();

        let config_file_path = format!("{}/config_test.yaml", tmp_path);
        std::fs::write(&config_file_path, CONFIG_SAMPLE)
            .expect("Failed to write config sample to temp file");

        let mut config_loader = ConfigLoader::with_path(config_file_path);
        let config_result = config_loader.load();
        println!("Config result: {:?}", config_result);
        assert!(config_result.is_ok());
        let _ = tmp_folder.close();
    }
}
