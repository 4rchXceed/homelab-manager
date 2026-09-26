use std::sync::Arc;

use log::info;
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::{RwLock, RwLockWriteGuard},
};
use tokio_rustls::{client::TlsStream, rustls::pki_types::ServerName};

use crate::{
    check_requirements::check_requirements,
    config::Config,
    context::AgentContext,
    db::database::Database,
    errors::{ClientInitError, ClientRuntimeError, NetInitError, NetRunError},
    file_client::rclone::sync_rclone,
    net::{
        auth::handle_server_auth, connection::create_tls_client, message_handler::MessageHandler,
        send_queue::SendQueue,
    },
};

pub struct Client {
    config: Config,
    socket: Option<Arc<RwLock<TlsStream<TcpStream>>>>,
    db: Database,
}

impl Client {
    pub async fn new(ini_path: String) -> Result<Self, ClientInitError> {
        let config = Config::from_ini(ini_path).map_err(|e| ClientInitError::ConfigError(e))?;

        check_requirements().map_err(|e| ClientInitError::RequirementMissing(e))?;

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

        info!("Authenticating to server...");

        let socket = self.socket.as_ref().ok_or(ClientRuntimeError::net_run(
            NetRunError::SocketClosedOrNotFound,
        ))?;

        handle_server_auth(socket.clone(), &self.config, &mut self.db).await?;

        info!("Authentication OK!");

        let fileserver_addr = format!(
            "{}:{}",
            self.config.server_host, self.config.file_server_port
        );

        let fileserver_auth = format!(
            "{}:{}",
            self.config.file_server_username, self.config.file_server_password
        );

        sync_rclone(
            fileserver_addr,
            self.config.service_folder.clone(),
            fileserver_auth,
            self.config.certificate_path.clone(),
        )
        .await
        .map_err(|e| ClientRuntimeError::FileSyncError(e))?;

        let context = AgentContext::new(self.db.clone(), self.config.clone());

        tokio::select! {
            _ = MessageHandler::handle_messages(socket.clone(), SendQueue::new(), context) => {},
            _ = tokio::signal::ctrl_c() => return Ok(()),
        }

        // if let Some(socket) = &mut self.socket {
        //     socket
        //         .write()
        //         .await
        //         .shutdown()
        //         .await
        //         .map_err(|e| ClientRuntimeError::net_run(NetRunError::SocketCloseError(e)))?;
        // }
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

        self.socket = Some(Arc::new(RwLock::new(tls_stream)));

        info!("Server connected!");

        return Ok(());
    }

    pub async fn recv(
        socket: &mut RwLockWriteGuard<'_, TlsStream<TcpStream>>,
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
