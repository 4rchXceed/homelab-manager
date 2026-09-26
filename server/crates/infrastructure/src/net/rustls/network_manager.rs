use std::sync::Arc;

use application::net::repositories::{
    net_connection::NetworkConnection, net_manager::NetworkManager,
};
use async_trait::async_trait;
use config::generic::net::NetConfig;
use tokio::{net::TcpListener, sync::RwLock};

use crate::{
    common::net::{create_connection::create_connection, errors::NetError},
    net::rustls::connection::RustTlsNetworkConnection,
};

pub struct RustlsNetworkManager {
    tcp_listener: Arc<RwLock<TcpListener>>,
    tcp_acceptor: Arc<tokio_rustls::TlsAcceptor>,
}

impl RustlsNetworkManager {
    pub async fn listen(port: usize, network_config: NetConfig) -> Result<Self, NetError> {
        let (acceptor, listener) = create_connection(port as u16, network_config).await?;

        return Ok(Self {
            tcp_listener: Arc::new(RwLock::new(listener)),
            tcp_acceptor: Arc::new(acceptor),
        });
    }
}

#[async_trait]
impl NetworkManager for RustlsNetworkManager {
    async fn accept_new(&self) -> Result<Arc<dyn NetworkConnection>, String> {
        let listener = self.tcp_listener.read().await;
        let acceptor = self.tcp_acceptor.clone();

        let (tcp_stream, _) = listener.accept().await.map_err(|e| e.to_string())?;

        let tls_stream = acceptor
            .accept(tcp_stream)
            .await
            .map_err(|e| e.to_string())?;

        let connection = RustTlsNetworkConnection::new(tls_stream);

        return Ok(Arc::new(connection));
    }
}
