use thiserror::Error;

#[derive(Debug, Error)]
pub enum VariableProviderError {
    #[error("Variable: 'string', missing property 'value'")]
    StringVarProviderMissingValue,
    #[error("Variable: 'user_var', error while getting value from db: {0}")]
    UserVarErrorWhileGettingValue(String),
    #[error("Variable: 'user_var', failed to ask user for value: {0}")]
    UserVarFailedToAskUser(String),
    #[error("Variable: 'user_var', failed to save value to db: {0}")]
    UserVarFailedToSaveValue(String),
    #[error("Variable: 'user_var', missing property '{0}'")]
    UserVarPropertyMissing(String),
}
