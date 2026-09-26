use std::sync::Arc;

use application::database::repositories::backups_db::BackupsDb;
use async_trait::async_trait;
use config::service::backup::config::BackupConfig;
use domain::backup::backup::Backup;
use log::trace;

use crate::{database::turso::connection::TursoDbConnection, sql};

pub struct TursoBackupsDb {
    conn: Arc<TursoDbConnection>,
}

impl TursoBackupsDb {
    pub fn new(conn: Arc<TursoDbConnection>) -> Self {
        return Self { conn };
    }
}

#[async_trait]
impl BackupsDb for TursoBackupsDb {
    async fn ensure_backups_for_service(
        &self,
        service_id: String,
        mut backup_configs: Vec<BackupConfig>,
    ) -> Result<Vec<Backup>, String> {
        let conn = self.conn.create_connection().map_err(|e| e.to_string())?;

        let mut rows = conn
            .query(sql!(get_backups_for_service), (service_id.clone(),))
            .await
            .map_err(|e| e.to_string())?;

        let mut backups = Vec::new();

        while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
            let backup_id = row.get::<String>(0).map_err(|e| e.to_string())?;
            let last_backup = row.get::<Option<u64>>(1).map_err(|e| e.to_string())?;

            let backup_config = backup_configs.iter().position(|c| c.id == backup_id);

            if let Some(index) = backup_config {
                let backup_config = backup_configs.remove(index);

                let backup = Backup::from_config_and_db(backup_config, last_backup);

                backups.push(backup);
            }
            // else...
            // Backup deleted, but we ignore this so if it's re-added it won't be lost.
        }

        // Add backups that are in the config but not in the database yet
        for backup_config in backup_configs {
            trace!(
                "Adding backup {} for service {} in the database",
                backup_config.id, service_id
            );

            conn.execute(
                sql!(insert_backup),
                (backup_config.id.clone(), service_id.clone()),
            )
            .await
            .map_err(|e| e.to_string())?;

            let backup = Backup::from_config_and_db(backup_config, None);

            backups.push(backup);
        }

        return Ok(backups);
    }
}
