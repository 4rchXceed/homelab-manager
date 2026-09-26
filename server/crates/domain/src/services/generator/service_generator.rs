use config::service::config_file::generator::config::{GeneratorArguments, GeneratorConfig};

pub struct ServiceGenerator {
    pub base: String,
    pub arguments: GeneratorArguments,
}

impl ServiceGenerator {
    pub fn from_config(config: &GeneratorConfig) -> Self {
        return ServiceGenerator {
            base: config.generator_id.clone(),
            arguments: config.args.clone(),
        };
    }
}
