use std::sync::Arc;

use crate::interactions::repositories::user_context::UserContext;

/// This WILL break the "clean architecture" pattern, but it's necessary to pass the required context to the VarProvider object.
/// This is made so the VarProviders can use, the database, for example.
/// So when a new varprovider is created, I may update this context to include the required context for the new varprovider.
pub struct VarProviderContext {
    pub user_context: Arc<dyn UserContext>,
}
