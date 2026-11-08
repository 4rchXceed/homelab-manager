use async_trait::async_trait;

#[async_trait]
pub trait UserVarsDb: Send + Sync {
    /// Returns:
    /// - Ok: Query successful
    ///     - Some: whether the variable exists with the id
    ///         - Some: the nullable value of the variable
    async fn get_value(&self, var_id: String) -> Result<Option<Option<String>>, String>;

    async fn save_value(&self, var_id: String, value: String) -> Result<(), String>;

    async fn get_dependencies(&self, var_id: String) -> Result<Vec<String>, String>;

    async fn save_dependencies(
        &self,
        var_id: String,
        dependencies: Vec<String>,
    ) -> Result<(), String>;
}
