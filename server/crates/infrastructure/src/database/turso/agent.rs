use application::database::repositories::agents_db::AgentsDb;
use async_trait::async_trait;
use turso::core::alloc::Arc;
use uuid::Uuid;

use crate::{database::turso::connection::TursoDbConnection, sql};

pub struct TursoAgentsDb {
    database_conn: Arc<TursoDbConnection>,
}

impl TursoAgentsDb {
    pub fn new(database_conn: Arc<TursoDbConnection>) -> Self {
        return Self { database_conn };
    }
}

#[async_trait]
impl AgentsDb for TursoAgentsDb {
    async fn get_reverse_api_key_for_agent(
        &self,
        agent_id: String,
    ) -> Result<Option<String>, String> {
        let conn = self
            .database_conn
            .create_connection()
            .map_err(|e| e.to_string())?;

        let mut rows = conn
            .query(sql!(get_reverse_api_key), (agent_id,))
            .await
            .map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let reverse_api_key = row.get::<String>(0).map_err(|e| e.to_string())?;

            return Ok(Some(reverse_api_key));
        } else {
            return Ok(None);
        }
    }

    async fn ensure_agent(&self, agent_id: String) -> Result<String, String> {
        let conn = self
            .database_conn
            .create_connection()
            .map_err(|e| e.to_string())?;

        let existing_agent = self.get_reverse_api_key_for_agent(agent_id.clone()).await?;

        if let Some(reverse_api_key) = existing_agent {
            return Ok(reverse_api_key);
        } else {
            let reverse_api_key = Uuid::new_v4().to_string();

            conn.execute(
                sql!(insert_agent),
                (agent_id.clone(), reverse_api_key.clone()),
            )
            .await
            .map_err(|e| e.to_string())?;

            return Ok(reverse_api_key);
        }
    }
}
