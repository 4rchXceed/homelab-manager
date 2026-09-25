use yaml_rust2::Yaml;

use crate::errors::ConfigError;
use consts::DEFAULT_BASH_GENERATOR_TIMEOUT;
use utils::config::parse_time;

#[derive(Debug, Clone)]
pub struct BashGeneratorConfig {
    pub commands: Vec<String>,
    pub timeout: usize,
}

impl BashGeneratorConfig {
    pub fn from_yaml(yaml: &Yaml) -> Result<Self, ConfigError> {
        let commands_yaml = yaml["commands"]
            .as_vec()
            .ok_or(ConfigError::GeneratorConfigBashCommandsMissing)?;
        let commands: Vec<String> = commands_yaml
            .iter()
            .map(|cmd| {
                cmd.as_str()
                    .ok_or(ConfigError::GeneratorConfigBashCommandNotString)
                    .map(|s| String::from(s))
            })
            .collect::<Result<Vec<String>, ConfigError>>()?;

        let timeout = parse_time(
            yaml["timeout"]
                .as_str()
                .unwrap_or(DEFAULT_BASH_GENERATOR_TIMEOUT),
        );
        if timeout.is_err() {
            return Err(ConfigError::GeneratorConfigBashTimeoutParseError(
                timeout.err().unwrap(),
            ));
        }
        let timeout = timeout.unwrap();

        return Ok(Self {
            commands: commands,
            timeout: timeout,
        });
    }
}

#[derive(Debug, Clone)]
pub enum GeneratorBaseConfig {
    Bash(BashGeneratorConfig),
    BashUnsandboxed(BashGeneratorConfig),
}

impl GeneratorBaseConfig {
    pub fn from_yaml(yaml: &Yaml, base: &str) -> Result<Self, ConfigError> {
        match base {
            "bash" => {
                let bash_config = BashGeneratorConfig::from_yaml(yaml)?;
                Ok(GeneratorBaseConfig::Bash(bash_config))
            }
            "bash-unsandboxed" => {
                let bash_config = BashGeneratorConfig::from_yaml(yaml)?;
                Ok(GeneratorBaseConfig::BashUnsandboxed(bash_config))
            }
            _ => Err(ConfigError::GeneratorConfigBaseMissing(base.to_string())),
        }
    }
}
