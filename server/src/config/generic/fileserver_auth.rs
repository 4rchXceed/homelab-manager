use yaml_rust2::Yaml;

use crate::config::config::ConfigError;

#[derive(Debug, Clone)]
pub struct FileServerAuthConfig {
    pub username: String,
    pub password: String,
}

impl FileServerAuthConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let username = yaml["username"]
            .as_str()
            .ok_or(ConfigError::FileServerAuthUsernameMissing)?;
        let password = yaml["password"]
            .as_str()
            .ok_or(ConfigError::FileServerAuthPasswordMissing)?;

        Ok(Self {
            username: String::from(username),
            password: String::from(password),
        })
    }
}
