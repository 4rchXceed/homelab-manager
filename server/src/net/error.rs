use crate::net::certgen::CertGenerationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to create TLS certificate: {0}")]
    TlsCertificateCreationError(CertGenerationError),
    #[error("Failed to create TLS configuration: {0}")]
    TlsConfigGenerationError(String),
    #[error("Failed to create TCP listener: {0}")]
    TcpListenerCreationError(String),
    #[error(
        "Failed to create TLS certificate directory: {0} (please check your /general/net/cert_dir configuration)"
    )]
    TlsCertificateDirCreationError(String),
}
