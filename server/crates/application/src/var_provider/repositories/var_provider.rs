use async_trait::async_trait;
use config::service::config_file::generator::config::VariableArguments;

use crate::var_provider::context::VarProviderContext;

#[async_trait]
pub trait VarProvider: Send + Sync {
    fn get_name(&self) -> String;

    async fn process(
        &self,
        args: VariableArguments,
        service_id: String,
        context: VarProviderContext,
    ) -> Result<String, String>;
}
