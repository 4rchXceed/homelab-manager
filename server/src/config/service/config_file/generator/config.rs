use yaml_rust2::Yaml;

use crate::config::{config::ConfigError, service::config_file::generator::arg::GeneratorArg};

#[derive(Debug)]
pub struct GeneratorConfig {
    pub generator_id: String,
    pub args: Vec<GeneratorArg>,
}

impl GeneratorConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let generator_id = yaml["generator"]
            .as_str()
            .ok_or(ConfigError::RunGeneratorIdMissing)?;

        if yaml["args"].is_badvalue() || !yaml["args"].is_array() {
            return Err(ConfigError::RunGeneratorArgsNotArray(
                generator_id.to_string(),
            ));
        }
        let args_raw = yaml["args"].as_vec().unwrap();

        let mut args = Vec::new();
        for arg_raw in args_raw.iter() {
            let arg = GeneratorArg::from_yaml(arg_raw);
            if arg.is_err() {
                return Err(arg.err().unwrap());
            }
            args.push(arg.unwrap());
        }

        return Ok(Self {
            generator_id: String::from(generator_id),
            args: args,
        });
    }
}
