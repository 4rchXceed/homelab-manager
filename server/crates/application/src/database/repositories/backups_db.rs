use async_trait::async_trait;
use config::service::backup::config::BackupConfig;
use domain::backup::backup::Backup;

#[async_trait]
pub trait BackupsDb: Send + Sync {
    async fn ensure_backups_for_service(
        &self,
        service_id: String,
        backup_configs: Vec<BackupConfig>,
    ) -> Result<Vec<Backup>, String>;
}
