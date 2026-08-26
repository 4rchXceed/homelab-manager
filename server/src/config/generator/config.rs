use yaml_rust2::Yaml;

use crate::config::{config::ConfigError, generator::base_config::GeneratorBaseConfig};

#[derive(Debug)]
pub struct GeneratorConfig {
    pub id: String,
    pub generator_base: GeneratorBaseConfig,
}

impl GeneratorConfig {
    pub fn from_yaml(yaml: &Yaml, key: String) -> Result<Self, ConfigError> {
        let base = yaml["base"]
            .as_str()
            .ok_or(ConfigError::GeneratorConfigBaseMissing(key.clone()))?;
        let generator_base = GeneratorBaseConfig::from_yaml(yaml, base);
        if generator_base.is_err() {
            return Err(generator_base.err().unwrap());
        }
        let generator_base = generator_base.unwrap();

        return Ok(Self {
            id: key,
            generator_base: generator_base,
        });
    }
}
