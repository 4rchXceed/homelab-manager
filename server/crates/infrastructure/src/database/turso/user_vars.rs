use std::sync::Arc;

use application::database::repositories::user_vars_db::UserVarsDb;
use async_trait::async_trait;
use turso::Connection;

use crate::sql;

use super::connection::TursoDbConnection;

pub struct TursoUserVarsDb {
    connection: Arc<TursoDbConnection>,
}

impl TursoUserVarsDb {
    pub fn new(connection: Arc<TursoDbConnection>) -> Self {
        Self { connection }
    }

    async fn id_from_id_str(
        &self,
        id_str: String,
        connection: &Connection,
    ) -> Result<Option<u64>, String> {
        let mut rows = connection
            .query(sql!(get_var_id), (id_str,))
            .await
            .map_err(|e| e.to_string())?;

        let first_row = rows.next().await.map_err(|e| e.to_string())?;

        if let Some(row) = first_row {
            let id: u64 = row.get(0).map_err(|e| e.to_string())?;

            return Ok(Some(id));
        } else {
            return Ok(None);
        }
    }

    async fn get_deps(
        &self,
        user_var_id: u64,
        connection: &Connection,
    ) -> Result<Vec<(String, u64)>, String> {
        let mut rows = connection
            .query(sql!(get_var_dependencies), (user_var_id,))
            .await
            .map_err(|e| e.to_string())?;

        let mut dependencies: Vec<(String, u64)> = Vec::new();

        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let dependency: String = row.get(0).map_err(|e| e.to_string())?;
            let id: u64 = row.get(1).map_err(|e| e.to_string())?;

            dependencies.push((dependency, id));
        }

        return Ok(dependencies);
    }
}

#[async_trait]
impl UserVarsDb for TursoUserVarsDb {
    async fn get_value(&self, var_id: String) -> Result<Option<Option<String>>, String> {
        let conn = self
            .connection
            .create_connection()
            .map_err(|e| e.to_string())?;

        let mut rows = conn
            .query(sql!(get_user_var_value), (var_id,))
            .await
            .map_err(|e| e.to_string())?;

        let first_row = rows.next().await.map_err(|e| e.to_string())?;

        if let Some(row) = first_row {
            let value: Option<String> = row.get(0).map_err(|e| e.to_string())?;

            return Ok(Some(value));
        } else {
            return Ok(None);
        }
    }

    async fn save_value(&self, var_id: String, value: String) -> Result<(), String> {
        let conn = self
            .connection
            .create_connection()
            .map_err(|e| e.to_string())?;

        if let Some(_) = self.get_value(var_id.clone()).await? {
            conn.execute(sql!(update_user_var_value), (value, var_id))
                .await
                .map_err(|e| e.to_string())?;

            return Ok(());
        } else {
            conn.execute(sql!(insert_user_var_value), (var_id, value))
                .await
                .map_err(|e| e.to_string())?;

            return Ok(());
        }
    }

    async fn get_dependencies(&self, var_id: String) -> Result<Vec<String>, String> {
        let conn = self
            .connection
            .create_connection()
            .map_err(|e| e.to_string())?;

        let id_op = self
            .id_from_id_str(var_id.clone(), &conn)
            .await
            .map_err(|e| format!("Failed to get var id: {}", e))?;

        let id = id_op.ok_or(format!("Var with id_str {} not found", var_id))?;

        let deps = self
            .get_deps(id, &conn)
            .await
            .map_err(|e| format!("Failed to get var dependencies: {}", e))?;

        return Ok(deps.into_iter().map(|(dep, _)| dep).collect());
    }

    async fn save_dependencies(
        &self,
        var_id: String,
        dependencies: Vec<String>,
    ) -> Result<(), String> {
        let conn = self
            .connection
            .create_connection()
            .map_err(|e| e.to_string())?;

        let variable_id = self
            .id_from_id_str(var_id.clone(), &conn)
            .await
            .map_err(|e| format!("Failed to get var id: {}", e))?
            .ok_or(format!("Var with id_str {} not found", var_id))?;

        let old_dependencies = self
            .get_deps(variable_id, &conn)
            .await
            .map_err(|e| format!("Failed to get old deps: {e}"))?;

        let old_dependencies_set: std::collections::HashSet<String> = old_dependencies
            .iter()
            .map(|(dep, _)| dep.clone())
            .collect();

        for service_id_str in &dependencies {
            if !old_dependencies_set.contains(service_id_str) {
                let service_id = self
                    .id_from_id_str(service_id_str.clone(), &conn)
                    .await
                    .map_err(|e| format!("Failed to get dep id: {}", e))?
                    .ok_or(format!("Service with id_str {} not found", service_id_str))?;

                conn.execute(sql!(add_var_dep), (service_id, variable_id))
                    .await
                    .map_err(|e| format!("Failed to add dependency: {}", e))?;
            }
        }

        for (old_service_id_str, old_service_id) in old_dependencies {
            if !dependencies.contains(&old_service_id_str) {
                conn.execute(sql!(rm_var_dep), (old_service_id, variable_id))
                    .await
                    .map_err(|e| format!("Failed to remove dependency: {}", e))?;
            }
        }

        return Ok(());
    }
}
