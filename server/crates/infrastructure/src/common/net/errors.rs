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

#[derive(Debug, Error)]
pub enum CertGenerationError {
    #[error("Failed to generate RSA key")]
    FailedToGenerateRSAKey,
    #[error("Failed to build cert name")]
    FailedToBuildName,
    #[error("Failed to build certificate")]
    FailedToBuildCertificate,
    #[error("Failed to set validity period")]
    FailedToSetValidityPeriod,
    #[error("Failed to set serial number")]
    FailedToSetSerialNumber,
    #[error("Failed to add valid IP addresses to the certificate")]
    FailedToAddValidIPAddresses,
    #[error("Failed to sign certificate")]
    FailedToSignCertificate,
    #[error("Failed to generate private key")]
    FailedToGeneratePrivateKey,
    #[error("Failed to write server private key")]
    FailedToWriteServerKey,
    #[error("Failed to generate rustls certificate")]
    FailedToGenerateCertificate,
    #[error("Failed to write server certificate")]
    FailedToWriteServerCertificate,
    #[error("Failed to write client certificate")]
    FailedToWriteClientCertificate,
    #[error("Failed to convert server certificate to private key: {0}")]
    FailedToConvertServerCertificateToPrivateKey(String),
    #[error("Failed to convert server certificate to PEM: {0}")]
    FailedToConvertServerCertificateToPem(String),
}
