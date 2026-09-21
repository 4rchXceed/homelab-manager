use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("Failed to open certificate file: {0}")]
    FailedToOpenCertificateFile(tokio_rustls::rustls::pki_types::pem::Error),
    #[error("Failed to add certificate to store: {0}")]
    FailedToAddCertificateToStore(tokio_rustls::rustls::Error),
}
