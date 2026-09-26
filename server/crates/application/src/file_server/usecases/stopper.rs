use crate::file_server::repositories::file_server::FileServerProvider;

pub struct FileServerStopper {
    file_server_provider: Box<dyn FileServerProvider>,
}

impl FileServerStopper {
    pub fn new(file_server_provider: Box<dyn FileServerProvider>) -> Self {
        return Self {
            file_server_provider,
        };
    }

    pub async fn stop(&self) -> Result<(), String> {
        self.file_server_provider.stop().await?;

        return Ok(());
    }
}
