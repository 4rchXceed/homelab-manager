use std::sync::Arc;

use application::{
    agent::repositories::agents_repository::AgentsRepository,
    net::repositories::{agent_connection::AgentConnection, net_manager::NetworkManager},
};
use async_trait::async_trait;
use config::generic::net::NetConfig;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio_rustls::server::TlsStream;

use crate::{
    common::net::{create_connection::create_connection, errors::NetError},
    net::rustls::{agent_connection::RustTlsAgentConnection, authenticator::TlsAgentAuthenticator},
};

pub struct RustlsNetworkManager {
    tcp_listener: Arc<RwLock<TcpListener>>,
    tcp_acceptor: Arc<tokio_rustls::TlsAcceptor>,
    agent_authenticator: Arc<TlsAgentAuthenticator>,
}

impl RustlsNetworkManager {
    pub async fn listen(
        port: usize,
        network_config: NetConfig,
        agent_repo: Arc<dyn AgentsRepository>,
    ) -> Result<Self, NetError> {
        let (acceptor, listener) = create_connection(port as u16, network_config).await?;

        return Ok(Self {
            tcp_listener: Arc::new(RwLock::new(listener)),
            tcp_acceptor: Arc::new(acceptor),
            agent_authenticator: Arc::new(TlsAgentAuthenticator::new(agent_repo)),
        });
    }

    async fn accept_new(&self) -> Result<TlsStream<TcpStream>, String> {
        let listener = self.tcp_listener.read().await;
        let acceptor = self.tcp_acceptor.clone();

        let (tcp_stream, _) = listener.accept().await.map_err(|e| e.to_string())?;

        let tls_stream = acceptor
            .accept(tcp_stream)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(tls_stream);
    }
}

#[async_trait]
impl NetworkManager for RustlsNetworkManager {
    async fn accept_new_auth_agent(&self) -> Result<Arc<dyn AgentConnection>, String> {
        let mut tls_stream = self.accept_new().await?;

        self.agent_authenticator
            .authenticate(&mut tls_stream)
            .await
            .map_err(|e| e.to_string())?;

        return Ok(Arc::new(RustTlsAgentConnection::new(tls_stream)));
    }
}
