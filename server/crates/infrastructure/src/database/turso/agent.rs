use std::collections::HashMap;

use application::database::repositories::agents_db::AgentsDb;
use async_trait::async_trait;
use domain::agent::{agent::Agent, update_agent::UpdateAgent};
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
    async fn get_agent_by_id(&self, id: String) -> Result<Option<Agent>, String> {
        let conn = self
            .database_conn
            .create_connection()
            .map_err(|e| e.to_string())?;

        let mut rows = conn
            .query(sql!(get_agent), (id.clone(),))
            .await
            .map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let api_key = row.get::<String>(0).map_err(|e| e.to_string())?;
            let ip = row.get::<String>(1).map_err(|e| e.to_string())?;
            let reverse_api_key = row.get::<String>(2).map_err(|e| e.to_string())?;

            return Ok(Some(Agent {
                id,
                api_key,
                ip,
                reverse_api_key,
                storages: Vec::new(),
                storages_configs: HashMap::new(),
            }));
        } else {
            return Ok(None);
        }
    }

    async fn ensure_agent(&self, agent: UpdateAgent) -> Result<(), String> {
        let conn = self
            .database_conn
            .create_connection()
            .map_err(|e| e.to_string())?;

        let existing_agent = self.get_agent_by_id(agent.id.clone()).await?;

        if existing_agent.is_none() {
            conn.execute(
                sql!(insert_agent),
                (
                    agent.id.clone(),
                    agent.api_key.clone(),
                    agent.ip.clone(),
                    Uuid::new_v4().to_string(),
                ),
            )
            .await
            .map_err(|e| e.to_string())?;
        } else if let Some(existing_agent) = existing_agent
            && (existing_agent.api_key != agent.api_key || existing_agent.ip != agent.ip)
        {
            conn.execute(
                sql!(update_agent),
                (agent.api_key.clone(), agent.ip.clone(), agent.id.clone()),
            )
            .await
            .map_err(|e| e.to_string())?;
        }

        return Ok(());
    }
}
