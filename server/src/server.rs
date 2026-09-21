use std::sync::{Arc, RwLock};

use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt},
    net::TcpListener,
    sync::Mutex,
};
use tokio_rustls::TlsAcceptor;

use crate::{
    config::{
        config::Config,
        loader::{ConfigLoadError, ConfigLoader},
    },
    context::{CommandContext, Context, LockedContext},
    database::generate_connection_pool,
    logger::{log_error, log_info},
    net::{connection::create_tls_connection, error::NetError},
    protocol::agent::{Agent, AgentLocked},
};

#[derive(Debug, Error)]
pub enum HomelabStartupFail {
    #[error("Failed to load configuration: {0}")]
    ConfigParseFail(ConfigLoadError),
    #[error("Failed to connect to the database")]
    DatabaseConnectionFail(r2d2::Error),
}

#[derive(Debug, Error)]
pub enum HomelabRuntimeError {
    #[error("Failed to start the agent server: {0}")]
    AgentServerStartupFail(NetError),
    #[error("Failed to accept connection from agent: {0}")]
    TcpListenerAcceptFail(std::io::Error),
    #[error("Failed to accept TLS connection from agent: {0}")]
    TlsAcceptFail(String),
    #[error("Failed to read server ID from agent: {0}")]
    TlsGetServerIDFailed(std::io::Error),
}

pub struct HomelabServer {
    config: Config,
    config_raw: String,
    agents: Vec<AgentLocked>,
    context: Arc<RwLock<Context>>,
}

impl HomelabServer {
    pub fn new() -> Result<Self, HomelabStartupFail> {
        let loader = ConfigLoader::new_from_env();
        let (config, raw_config) = loader
            .load()
            .map_err(|e| HomelabStartupFail::ConfigParseFail(e))?;

        let pool = generate_connection_pool()
            .map_err(|e| HomelabStartupFail::DatabaseConnectionFail(e))?;

        let context = Arc::new(RwLock::new(Context {
            config: config.clone(),
            db_pool: pool,
        }));

        let agents = config
            .servers_config
            .iter()
            .map(|s| {
                Agent::new(
                    s.id.clone(),
                    s.api_key.clone(),
                    &config.clone(),
                    LockedContext::new_server(context.clone()),
                )
            })
            .filter_map(|v| {
                if let Ok(agent) = v {
                    return Some(agent);
                } else if let Err(e) = v {
                    log_error(
                        format!("Failed to create agent: {:?}", e).as_str(),
                        &CommandContext::server_logger(),
                    );
                }
                return None;
            })
            .collect();

        return Ok(Self {
            config: config,
            config_raw: raw_config,
            agents: agents,
            context: context,
        });
    }

    pub async fn run(&mut self) -> Result<(), HomelabRuntimeError> {
        let (agent_listener, agent_acceptor) = self.generate_agent_socket().await?;

        log_info("Server started!", &CommandContext::server_logger());

        loop {
            tokio::select! {
                res = agent_listener.accept() => {
                    if let Err(e) = self.accept_agent(res, agent_acceptor.clone()).await {
                        log_error(
                            format!("Failed to accept agent connection: {:?}", e).as_str(),
                            &CommandContext::server_logger(),
                        );
                    }
                }
            }
        }
    }

    pub async fn accept_agent(
        &mut self,
        res: Result<(tokio::net::TcpStream, std::net::SocketAddr), std::io::Error>,
        acceptor: TlsAcceptor,
    ) -> Result<(), HomelabRuntimeError> {
        let (stream, _) = res.map_err(|e| HomelabRuntimeError::TcpListenerAcceptFail(e))?;

        let mut stream_accepted = acceptor
            .accept(stream)
            .await
            .map_err(|e| HomelabRuntimeError::TlsAcceptFail(e.to_string()))?;

        let mut server_id = Vec::new();

        stream_accepted
            .read_until(b'\n', &mut server_id)
            .await
            .map_err(|e| HomelabRuntimeError::TlsGetServerIDFailed(e))?;

        let server_id_str = String::from_utf8_lossy(&server_id).trim().to_string();

        let agent_opt = self.agents.iter().find(|agent| {
            if let Ok(agent_locked) = agent.read() {
                agent_locked.get_id() == server_id_str
            } else {
                false
            }
        });

        if let Some(agent_locked) = agent_opt {
            if let Err(e) =
                Agent::connect(Arc::new(Mutex::new(stream_accepted)), agent_locked.clone()).await
            {
                log_error(
                    format!(
                        "Failed to connect agent with server ID {}: {:?}",
                        server_id_str, e
                    )
                    .as_str(),
                    &CommandContext::server_logger(),
                );
            }
        } else {
            log_error(
                format!(
                    "Received connection from unknown server ID: {}",
                    server_id_str
                )
                .as_str(),
                &CommandContext::server_logger(),
            );
        }

        return Ok(());
    }

    pub async fn generate_agent_socket(
        &mut self,
    ) -> Result<(TcpListener, TlsAcceptor), HomelabRuntimeError> {
        let net_connection = create_tls_connection(
            self.config.config_general.net_config.server_port,
            &self.config.config_general.net_config,
        )
        .await
        .map_err(|e| HomelabRuntimeError::AgentServerStartupFail(e))?;

        return Ok(net_connection);
    }

    pub fn run_cli_socket(&mut self) {
        // TODO
    }

    pub fn reload(&mut self, context: LockedContext) {}
}
