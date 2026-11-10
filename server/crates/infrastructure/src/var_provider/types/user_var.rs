use std::sync::Arc;

use application::{
    database::repositories::user_vars_db::UserVarsDb,
    var_provider::{context::VarProviderContext, repositories::var_provider::VarProvider},
};
use async_trait::async_trait;
use config::service::config_file::generator::config::VariableArguments;

use crate::var_provider::types::errors::VariableProviderError;

pub struct UserVarProvider {
    user_vars_db: Arc<dyn UserVarsDb>,
}

impl UserVarProvider {
    pub fn new(user_vars_db: Arc<dyn UserVarsDb>) -> Self {
        Self { user_vars_db }
    }
}

#[async_trait]
impl VarProvider for UserVarProvider {
    fn get_name(&self) -> String {
        return String::from("user_var");
    }

    async fn process(
        &self,
        args: VariableArguments,
        _service_id: String,
        context: VarProviderContext,
    ) -> Result<String, String> {
        if let Some(var_id) = args["id"].as_str() {
            if let Some(var_name) = args["name"].as_str() {
                let default_value = args["default_value"].as_str().map(|s| s.to_string());

                let var_value_op = self
                    .user_vars_db
                    .get_value(var_id.to_string())
                    .await
                    .map_err(|e| {
                        VariableProviderError::UserVarErrorWhileGettingValue(e).to_string()
                    })?;

                // Return the already-existing value
                if let Some(var_value) = var_value_op {
                    return Ok(var_value);
                } else {
                    // Return the default value if it exists
                    if let Some(default_value) = default_value {
                        let var_get_result = self
                            .user_vars_db
                            .get_value(var_id.to_string())
                            .await
                            .map_err(|e| {
                                VariableProviderError::UserVarErrorWhileGettingValue(e).to_string()
                            })?;

                        if var_get_result.is_none() {
                            // Save the value to the database
                            self.user_vars_db
                                .save_value(var_id.to_string(), default_value.clone())
                                .await
                                .map_err(|e| {
                                    VariableProviderError::UserVarFailedToSaveValue(e).to_string()
                                })?;
                        }

                        return Ok(default_value);
                    } else {
                        // Ask user if no value exists
                        let variable_content = context
                            .user_context
                            .prompt(&format!("Value of the variable: {}: ", var_name))
                            .await
                            .map_err(|e| {
                                VariableProviderError::UserVarFailedToAskUser(e).to_string()
                            })?;

                        // Save the value to the database
                        self.user_vars_db
                            .save_value(var_id.to_string(), variable_content.clone())
                            .await
                            .map_err(|e| {
                                VariableProviderError::UserVarFailedToSaveValue(e).to_string()
                            })?;

                        // Return the value
                        return Ok(variable_content);
                    }
                }
            } else {
                return Err(
                    VariableProviderError::UserVarPropertyMissing(String::from("name")).to_string(),
                );
            }
        } else {
            return Err(
                VariableProviderError::UserVarPropertyMissing(String::from("id")).to_string(),
            );
        }
    }
}
