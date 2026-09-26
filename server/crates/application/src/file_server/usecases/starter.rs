use std::sync::Arc;

use consts::DEFAULT_FILE_SERVER_TIMEOUT;

use crate::file_server::repositories::file_server::FileServerProvider;

pub struct FileServerStarter {
    file_server_provider: Arc<dyn FileServerProvider>,
}

impl FileServerStarter {
    pub fn new(file_server_provider: Arc<dyn FileServerProvider>) -> Self {
        return Self {
            file_server_provider,
        };
    }

    pub async fn start_and_wait(&self) -> Result<(), String> {
        self.file_server_provider.start_threaded().await?;

        self.file_server_provider
            .wait_until_started(DEFAULT_FILE_SERVER_TIMEOUT)
            .await?;

        return Ok(());
    }
}
