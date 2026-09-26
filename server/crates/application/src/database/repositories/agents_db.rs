use async_trait::async_trait;

#[async_trait]
pub trait AgentsDb: Send + Sync {
    async fn get_reverse_api_key_for_agent(
        &self,
        agent_id: String,
    ) -> Result<Option<String>, String>;

    async fn ensure_agent(&self, agent_id: String) -> Result<String, String>;
}
