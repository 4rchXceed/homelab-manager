use std::time::Duration;

use log::{SetLoggerError, info, trace};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::sleep,
};
use tokio_rustls::{client::TlsStream, rustls::pki_types::ServerName};

use crate::{
    config::{Config, ConfigError},
    db::database::{Database, DatabaseOpenError},
    errors::{AuthError, ClientRuntimeError, NetInitError, NetRunError},
    net::connection::create_tls_client,
};

#[derive(Debug, Error)]
pub enum ClientInitError {
    #[error("Configuration error: {0}")]
    ConfigError(ConfigError),
    #[error("Failed to open database: {0} (path: {1}")]
    OpenDbError(DatabaseOpenError, String),
    #[error("Failed to create logger (simple_logger): {0}")]
    LogInitError(SetLoggerError),
    #[error("Invalid log level")]
    LogLevelError,
}

pub struct Client {
    config: Config,
    socket: Option<TlsStream<TcpStream>>,
    db: Database,
}

impl Client {
    pub async fn new(ini_path: String) -> Result<Self, ClientInitError> {
        let config = Config::from_ini(ini_path).map_err(|e| ClientInitError::ConfigError(e))?;

        let database = Database::load(config.database_path.clone())
            .await
            .map_err(|e| ClientInitError::OpenDbError(e, config.database_path.clone()))?;

        simple_logger::init_with_level(match config.log_level.to_lowercase().as_str() {
            "debug" => log::Level::Debug,
            "error" => log::Level::Error,
            "trace" => log::Level::Trace,
            "warn" => log::Level::Warn,
            "info" => log::Level::Info,
            _ => return Err(ClientInitError::LogLevelError),
        })
        .map_err(|e| ClientInitError::LogInitError(e))?;

        return Ok(Client {
            config: config,
            socket: None,
            db: database,
        });
    }

    pub async fn run(&mut self) -> Result<(), ClientRuntimeError> {
        info!("Server starting...");

        self.connect_to_server().await?;

        self.handle_server_auth().await?;

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(1)) => {},
                _ = tokio::signal::ctrl_c() => break,
            }
        }

        if let Some(socket) = &mut self.socket {
            socket
                .shutdown()
                .await
                .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketCloseError(e)))?;
        }
        return Ok(());
    }

    pub async fn connect_to_server(&mut self) -> Result<(), ClientRuntimeError> {
        info!("Connecting to server");

        let connector = create_tls_client(self.config.certificate_path.clone())
            .map_err(|e| ClientRuntimeError::net_init(NetInitError::TlsCreationError(e)))?;

        let addr = format!("{}:{}", self.config.server_host, self.config.server_port);

        let stream = TcpStream::connect(addr.clone()).await.map_err(|e| {
            ClientRuntimeError::net_init(NetInitError::ServerConnectionError(e, addr))
        })?;

        let domain = ServerName::try_from(self.config.server_host.clone())
            .map_err(|_| ClientRuntimeError::net_init(NetInitError::InvalidServerHost))?;

        let tls_stream = connector
            .connect(domain, stream)
            .await
            .map_err(|e| ClientRuntimeError::net_init(NetInitError::TlsError(e.to_string())))?;

        self.socket = Some(tls_stream);

        info!("Server connected!");

        return Ok(());
    }

    async fn handle_server_auth(&mut self) -> Result<(), ClientRuntimeError> {
        info!("Authenticating to server...");

        let socket = self.socket.as_mut().ok_or(ClientRuntimeError::net_run(
            NetRunError::SocketClosedOrNotFound,
        ))?;

        trace!("Starting auth protocol");

        let mut id = self.config.id.clone();
        id.push_str("\n");

        trace!("[OK] Sending id: {id}");

        socket
            .write(id.as_bytes())
            .await
            .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

        if let ack = Self::recv(socket, 4).await?
            && ack != b"AUTH"
        {
            let str = String::from_utf8_lossy(ack.as_slice());

            trace!("[ERROR] AUTH ACK NOK! Got: {str}");

            return Err(ClientRuntimeError::auth(AuthError::AuthAckNotSent));
        }

        trace!("[OK] AUTH ACK");

        let api_key_newline = self.config.api_key.clone() + "\n";

        socket
            .write(api_key_newline.as_bytes())
            .await
            .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

        let auth_ack = Self::recv(socket, 2).await?;

        if auth_ack != b"OK" {
            let str = String::from_utf8_lossy(&auth_ack);

            trace!("[ERROR]: API KEY NOT OK! Got: {str}");

            return Err(ClientRuntimeError::auth(AuthError::InvalidApiKey));
        }

        trace!("[OK] API KEY OK");

        let reverse_api_key = Self::recv(socket, 36).await?;

        let reverse_api_key_str = String::from_utf8(reverse_api_key)
            .map_err(|_| ClientRuntimeError::auth(AuthError::ReverseApiKeyIsntUtf8))?;

        if let Some(correct_reverse_api_key) = &self.db.reverse_api_key {
            trace!("[OK] API KEY FOUND (for security reasons, the api key isn't shown)");
            if &reverse_api_key_str != correct_reverse_api_key {
                trace!("[ERROR] COULD NOT AUTH SERVER");

                socket
                    .write(b"ER")
                    .await
                    .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

                return Err(ClientRuntimeError::auth(AuthError::InvalidReverseApiKey));
            }
        } else {
            trace!("First time receiving api key! Adding it to the db");

            self.db.reverse_api_key = Some(reverse_api_key_str);

            trace!("Saving the database");

            self.db
                .save(self.config.database_path.clone())
                .await
                .map_err(|e| ClientRuntimeError::auth(AuthError::NewReverseApiKeyDbSave(e)))?;
        }

        socket
            .write(b"OK")
            .await
            .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketWriteError(e)))?;

        trace!("Finished auth protocol");

        info!("Ok!");

        return Ok(());
    }

    async fn recv(
        socket: &mut TlsStream<TcpStream>,
        length: usize,
    ) -> Result<Vec<u8>, ClientRuntimeError> {
        let mut buffer = vec![0; length];

        socket
            .read(&mut buffer)
            .await
            .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketError(e)))?;

        return Ok(buffer);
    }
}
