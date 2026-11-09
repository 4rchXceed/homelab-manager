use async_trait::async_trait;
use config::generator::config::GeneratorBaseConfig;
use domain::agent::protocol::message::GeneratorMode;

#[async_trait]
pub trait GeneratorBase: Send + Sync {
    fn get_name(&self) -> &str;
    /// Returns:
    /// - Vec<String>: the commands
    async fn generate_commands(
        &self,
        config: &GeneratorBaseConfig,
        arguments: &Vec<String>,
        file_path: &String,
    ) -> Result<(Vec<String>, GeneratorMode, Option<String>), String>;
}
