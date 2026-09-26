use std::sync::Arc;

use application::database::repositories::{backups_db::BackupsDb, services_db::ServicesDb};
use async_trait::async_trait;
use config::service::config::ServiceConfig;
use domain::services::{service::Service, sync_config::SyncConfig};
use log::trace;
use turso::{Connection, Row};

use crate::{database::turso::connection::TursoDbConnection, sql};

pub struct TursoServicesDb {
    connection: Arc<TursoDbConnection>,
    backups_db: Arc<dyn BackupsDb>,
}

impl TursoServicesDb {
    pub fn new(connection: Arc<TursoDbConnection>, backups_db: Arc<dyn BackupsDb>) -> Self {
        return Self {
            connection,
            backups_db,
        };
    }

    async fn process_service_row(
        &self,
        row: Row,
        service: ServiceConfig,
    ) -> Result<Service, String> {
        let agent_host_id = row.get::<Option<String>>(2).map_err(|e| e.to_string())?;
        let last_sync = row.get::<Option<i64>>(0).map_err(|e| e.to_string())?;
        let sync_time = row.get::<Option<i64>>(1).map_err(|e| e.to_string())?;
        let sync_agent_id = row.get::<Option<String>>(3).map_err(|e| e.to_string())?;
        let sync_storage_id = row.get::<Option<String>>(4).map_err(|e| e.to_string())?;

        let sync_config = if let Some(sync_time) = sync_time
            && let Some(sync_agent_id) = sync_agent_id
            && let Some(sync_storage_id) = sync_storage_id
        {
            Some(SyncConfig {
                last_sync: last_sync,
                sync_agent_id: sync_agent_id,
                sync_agent_storage_id: sync_storage_id,
                sync_time: sync_time,
            })
        } else {
            None
        };

        let backups = self
            .backups_db
            .ensure_backups_for_service(service.id.clone(), service.backup_configs.clone())
            .await?;

        let service = Service::from_config_and_db(service, agent_host_id, sync_config, backups);

        return Ok(service);
    }

    async fn insert_service(
        &self,
        conn: Connection,
        service: ServiceConfig,
    ) -> Result<Service, String> {
        conn.execute(sql!(insert_service), (service.id.clone(),))
            .await
            .map_err(|e| e.to_string())?;

        let backups = self
            .backups_db
            .ensure_backups_for_service(service.id.clone(), service.backup_configs.clone())
            .await?;

        let service = Service::from_config_and_db(service, None, None, backups);

        return Ok(service);
    }
}

#[async_trait]
impl ServicesDb for TursoServicesDb {
    async fn ensure_service(&self, service: ServiceConfig) -> Result<Service, String> {
        trace!("Ensuring services: {} into database", service.id);

        let conn = self
            .connection
            .create_connection()
            .map_err(|e| e.to_string())?;

        let mut rows = conn
            .query(sql!(get_service_fields), (service.id.clone(),))
            .await
            .map_err(|e| e.to_string())?;

        let service = if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            trace!("Service already exists in database");
            self.process_service_row(row, service).await?
        } else {
            trace!("Service does not exist in database, inserting");
            self.insert_service(conn, service).await?
        };

        return Ok(service);
    }
}
