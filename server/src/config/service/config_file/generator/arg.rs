use yaml_rust2::Yaml;

use crate::config::config::ConfigError;

#[derive(Debug, Clone)]
pub struct UserVarConfig {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub enum GeneratorArg {
    String(String),
    UserVar(UserVarConfig),
}

impl GeneratorArg {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let argument_type = yaml["type"]
            .as_str()
            .ok_or(ConfigError::GeneratorArgumentTypeNotFound)?;
        match argument_type {
            "string" => {
                let value = yaml["value"]
                    .as_str()
                    .ok_or(ConfigError::GeneratorArgumentStringNotFound)?;
                return Ok(GeneratorArg::String(value.to_string()));
            }
            "user_var" => {
                let name = yaml["name"]
                    .as_str()
                    .ok_or(ConfigError::GeneratorArgumentUserVarNameNotFound)?;
                let id = yaml["id"]
                    .as_str()
                    .ok_or(ConfigError::GeneratorArgumentUserVarIdNotFound)?;
                return Ok(GeneratorArg::UserVar(UserVarConfig {
                    name: name.to_string(),
                    id: id.to_string(),
                }));
            }
            _ => {
                return Err(ConfigError::GeneratorArgumentTypeNotSupported(
                    String::from(argument_type),
                ));
            }
        }
    }
}
