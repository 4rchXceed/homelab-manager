pub struct SyncConfig {
    pub last_sync: Option<i64>,
    pub sync_time: i64,

    pub sync_agent_id: String,
    pub sync_agent_storage_id: String,
}
