use std::{sync::Arc, time::Duration};

use application::file_server::repositories::file_server::FileServerProvider;
use async_trait::async_trait;
use config::generic::{
    config::GeneralConfig, fileserver_auth::FileServerAuthConfig, net::NetConfig,
};
use consts::DEFAULT_FILE_SERVER_LOCAL_PORT;
use log::{info, trace};
use openssl::base64;
use tokio::{
    process::{Child, Command},
    sync::RwLock,
    task::JoinHandle,
    time::sleep,
};

use crate::file_server::rclone::{
    errors::FileServerErrors, http_to_https::proxy::HttpToHttpsProxy,
};

pub struct RCloneFileServer {
    path: String,
    auth: FileServerAuthConfig,
    port: u16,
    process: Arc<RwLock<Option<Child>>>,
    thread_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    config: NetConfig,
}

impl RCloneFileServer {
    pub fn new(config: &GeneralConfig) -> Self {
        return Self {
            path: config.services_folder.clone(),
            auth: config.file_server_auth.clone(),
            port: config.net_config.file_server_port as u16,
            process: Arc::new(RwLock::new(None)),
            config: config.net_config.clone(),
            thread_handle: Arc::new(RwLock::new(None)),
        };
    }

    async fn start_proxy(&self) -> Result<(), String> {
        if self.thread_handle.read().await.is_some() {
            return Err(FileServerErrors::ProxyAlreadyStarted.to_string());
        }
        let mut proxy = HttpToHttpsProxy::new(
            DEFAULT_FILE_SERVER_LOCAL_PORT,
            self.port,
            self.config.clone(),
        )
        .await?;

        let thread = tokio::spawn(async move {
            proxy.start();
        });

        let mut thread_lock = self.thread_handle.write().await;
        *thread_lock = Some(thread);

        return Ok(());
    }
}

#[async_trait]
impl FileServerProvider for RCloneFileServer {
    async fn start_threaded(&self) -> Result<(), String> {
        self.start_proxy().await?;

        let mut command = Command::new("rclone");

        command.arg("serve");
        command.arg("http");
        command.arg("--user").arg(self.auth.username.clone());
        command.arg("--pass").arg(self.auth.password.clone());
        command
            .arg("--addr")
            .arg(format!("localhost:{}", DEFAULT_FILE_SERVER_LOCAL_PORT));
        command.arg(self.path.clone());

        let child = command.spawn().map_err(|e| e.to_string())?;

        let mut process_lock = self.process.write().await;
        *process_lock = Some(child);

        return Ok(());
    }

    async fn stop(&self) -> Result<(), String> {
        let mut process_lock = self.process.write().await;

        if let Some(child) = process_lock.as_mut() {
            child.kill().await.map_err(|e| e.to_string())?;
            *process_lock = None;
        }

        return Ok(());
    }

    async fn wait_until_started(&self, timeout: Duration) -> Result<(), String> {
        trace!("Waiting for file server to start...");
        // Disable cert verif
        let config = ureq::tls::TlsConfig::builder()
            .disable_verification(true)
            .build();

        let agent: ureq::Agent = ureq::Agent::config_builder()
            .tls_config(config)
            .build()
            .into();

        let auth = format!("{}:{}", self.auth.username, self.auth.password);
        let value = format!("Basic {}", base64::encode_block(auth.as_bytes()));

        let address = format!("https://127.0.0.1:{}", self.port);
        let mut elapsed = Duration::new(0, 0);

        loop {
            if elapsed > timeout {
                return Err(FileServerErrors::TimeoutError.to_string());
            }

            trace!("Checking if file server is started...");

            let result = agent
                .get(address.clone())
                .header("Authorization", value.clone())
                .call();

            if let Err(e) = result {
                trace!("File server not started yet: {e}");
            } else {
                info!("File server started!");
                return Ok(());
            }

            sleep(Duration::from_millis(500)).await;
            elapsed += Duration::from_millis(500);
        }
    }
}
