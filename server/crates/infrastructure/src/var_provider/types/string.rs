use application::var_provider::{
    context::VarProviderContext, repositories::var_provider::VarProvider,
};
use async_trait::async_trait;
use config::service::config_file::generator::config::VariableArguments;

use crate::var_provider::types::errors::VariableProviderError;

pub struct StringVarProvider {}

impl StringVarProvider {
    pub fn new() -> Self {
        return Self {};
    }
}

#[async_trait]
impl VarProvider for StringVarProvider {
    fn get_name(&self) -> String {
        return String::from("string");
    }

    async fn process(
        &self,
        args: VariableArguments,
        _service_id: String,
        _context: VarProviderContext,
    ) -> Result<String, String> {
        if let Some(value) = args["value"].as_str() {
            return Ok(value.to_string());
        } else {
            return Err(VariableProviderError::StringVarProviderMissingValue.to_string());
        }
    }
}
