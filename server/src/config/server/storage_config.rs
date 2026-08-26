#[derive(Debug)]
pub struct StorageConfig {
    pub path: String,
    pub fallback: String,
    pub do_not_create: bool,
}
