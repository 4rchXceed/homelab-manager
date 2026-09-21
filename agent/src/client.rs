use thiserror::Error;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_rustls::{client::TlsStream, rustls::pki_types::ServerName};

use crate::{
    config::{Config, ConfigError},
    net::connection::create_tls_client,
};

#[derive(Debug, Error)]
pub enum ClientInitError {
    #[error("Configuration error: {0}")]
    ConfigError(ConfigError),
}

#[derive(Debug, Error)]
pub enum ClientRuntimeError {
    #[error("Network error: {0}")]
    NetworkError(#[from] crate::net::error::NetError),
    #[error("Invalid server host")]
    InvalidServerHost,
    #[error("Server connection error: {0}")]
    ServerConnectionError(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    TlsError(String),
    #[error("Socket close error: {0}")]
    SocketCloseError(std::io::Error),
}

pub struct Client {
    config: Config,
    socket: Option<TlsStream<TcpStream>>,
}

impl Client {
    pub fn new(ini_path: String) -> Result<Self, ClientInitError> {
        let config = Config::from_ini(ini_path).map_err(|e| ClientInitError::ConfigError(e))?;

        return Ok(Client {
            config: config,
            socket: None,
        });
    }

    pub async fn run(&mut self) -> Result<(), ClientRuntimeError> {
        self.connect_to_server().await?;
        //...
        if let Some(socket) = &mut self.socket {
            socket
                .shutdown()
                .await
                .map_err(|e| ClientRuntimeError::SocketCloseError(e))?;
        }
        return Ok(());
    }

    pub async fn connect_to_server(&mut self) -> Result<(), ClientRuntimeError> {
        let connector = create_tls_client(self.config.certificate_path.clone())
            .map_err(|e| ClientRuntimeError::NetworkError(e))?;

        let addr = format!("{}:{}", self.config.server_host, self.config.server_port);

        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| ClientRuntimeError::ServerConnectionError(e))?;

        let domain = ServerName::try_from(self.config.server_host.clone())
            .map_err(|_| ClientRuntimeError::InvalidServerHost)?;

        let tls_stream = connector
            .connect(domain, stream)
            .await
            .map_err(|e| ClientRuntimeError::TlsError(e.to_string()))?;

        self.socket = Some(tls_stream);

        return Ok(());
    }
}
