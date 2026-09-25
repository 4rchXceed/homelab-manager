use std::sync::Arc;

use application::net::repositories::{
    net_connection::NetworkConnection, net_manager::NetworkManager,
};
use async_trait::async_trait;
use config::generic::net::NetConfig;
use tokio::{net::TcpListener, sync::RwLock};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    },
};

use crate::net::rustls::{
    certgen::{generate_ca, generate_server_leaf},
    connection::RustTlsNetworkConnection,
    errors::{CertGenerationError, NetError},
};

pub struct RustlsNetworkManager {
    tcp_listener: Arc<RwLock<TcpListener>>,
    tcp_acceptor: Arc<tokio_rustls::TlsAcceptor>,
}

impl RustlsNetworkManager {
    pub async fn listen(port: usize, network_config: NetConfig) -> Result<Self, NetError> {
        let addr = format!("0.0.0.0:{}", port);

        if !std::path::Path::new(&network_config.cert_dir).exists() {
            std::fs::create_dir_all(&network_config.cert_dir)
                .map_err(|e| NetError::TlsCertificateDirCreationError(e.to_string()))?;
        }

        let server_cert_path = format!("{}/server.crt", network_config.cert_dir);
        let key_path = format!("{}/server.key", network_config.cert_dir);
        let client_crt_path = format!("{}/client.crt", network_config.cert_dir);

        Self::generate_certs(
            server_cert_path.clone(),
            key_path.clone(),
            client_crt_path,
            network_config.binds.iter().map(|e| e.to_string()).collect(),
        )
        .map_err(|e| NetError::TlsCertificateCreationError(e))?;

        let config = Arc::new(
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(
                    vec![
                        CertificateDer::from_pem_file(server_cert_path)
                            .map_err(|e| NetError::TlsConfigGenerationError(e.to_string()))?,
                    ],
                    PrivateKeyDer::from_pem_file(key_path)
                        .map_err(|e| NetError::TlsConfigGenerationError(e.to_string()))?,
                )
                .map_err(|e| NetError::TlsConfigGenerationError(e.to_string()))?,
        );

        let acceptor = TlsAcceptor::from(config.clone());
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| NetError::TcpListenerCreationError(e.to_string()))?;

        return Ok(Self {
            tcp_listener: Arc::new(RwLock::new(listener)),
            tcp_acceptor: Arc::new(acceptor),
        });
    }

    fn generate_certs(
        server_cert_path: String,
        key_path: String,
        client_crt_path: String,
        bind_ips: Vec<String>,
    ) -> Result<(), CertGenerationError> {
        if std::path::Path::new(&server_cert_path).exists()
            && std::path::Path::new(&key_path).exists()
            && std::path::Path::new(&client_crt_path).exists()
        {
            return Ok(());
        }

        let (ca_cert, ca_key) = generate_ca()?;
        let (server_cert, server_key) = generate_server_leaf(&bind_ips, &ca_cert, &ca_key)?;

        let cert_pem = server_cert.to_pem().map_err(|e| {
            CertGenerationError::FailedToConvertServerCertificateToPem(e.to_string())
        })?;

        let ca_cert_pem = ca_cert.to_pem().map_err(|e| {
            CertGenerationError::FailedToConvertServerCertificateToPem(e.to_string())
        })?;

        let key_pem = server_key.private_key_to_pem_pkcs8().map_err(|e| {
            CertGenerationError::FailedToConvertServerCertificateToPrivateKey(e.to_string())
        })?;

        std::fs::write(&server_cert_path, cert_pem)
            .map_err(|_| CertGenerationError::FailedToWriteServerCertificate)?;
        std::fs::write(&key_path, key_pem)
            .map_err(|_| CertGenerationError::FailedToWriteServerKey)?;
        std::fs::write(&client_crt_path, ca_cert_pem)
            .map_err(|_| CertGenerationError::FailedToWriteClientCertificate)?;
        return Ok(());
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
