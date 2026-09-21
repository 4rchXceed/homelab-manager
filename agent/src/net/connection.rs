use tokio_rustls::{
    TlsConnector,
    rustls::{
        ClientConfig,
        pki_types::{CertificateDer, pem::PemObject},
    },
};

use crate::net::error::NetError;

pub fn create_tls_client(cert_path: String) -> Result<TlsConnector, NetError> {
    let certificate = CertificateDer::from_pem_file(cert_path)
        .map_err(|e| NetError::FailedToOpenCertificateFile(e))?;

    let mut root_store = tokio_rustls::rustls::RootCertStore::empty();
    root_store
        .add(certificate)
        .map_err(|e| NetError::FailedToAddCertificateToStore(e))?;

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(std::sync::Arc::new(config));
    return Ok(connector);
}
