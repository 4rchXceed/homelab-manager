use yaml_rust2::Yaml;

use crate::config::config::ConfigError;

#[derive(Debug)]
pub enum GeneratorArg {
    String(String),
    // UserVar(UserVarConfig), TODO
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
            _ => {
                return Err(ConfigError::GeneratorArgumentTypeNotSupported(
                    String::from(argument_type),
                ));
            }
        }
    }
}
