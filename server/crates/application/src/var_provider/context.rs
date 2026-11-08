use std::sync::Arc;

use crate::{
    database::repositories::user_vars_db::UserVarsDb,
    interactions::repositories::user_context::UserContext,
};

/// This WILL break the "clean architecture" pattern, but it's necessary to pass the required context to the VarProvider object.
/// This is made so the VarProviders can use, the database, for example.
/// So when a new varprovider is created, I may update this context to include the required context for the new varprovider.
pub struct VarProviderAppContext {
    pub user_context: Arc<dyn UserContext>,
    pub user_vars_db: Arc<dyn UserVarsDb>,
}
