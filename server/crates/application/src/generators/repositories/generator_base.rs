use config::service::config_file::generator::config::GeneratorArguments;
use domain::agent::protocol::message::GeneratorMode;

pub trait GeneratorBase: Send + Sync {
    fn get_name(&self) -> &str;
    /// Returns:
    /// - Vec<String>: the commands
    fn generate_commands(
        &self,
        config: &GeneratorArguments,
    ) -> Result<(Vec<String>, GeneratorMode), String>;
}
