pub enum HLMCommunicationError {
    CertFileReadError(rustls::pki_types::pem::Error),
    CertFileNotAdded(rustls::Error),
    MissingKeyPath,
    KeyFileReadError(rustls::pki_types::pem::Error),
    CouldNotConfigCert,
}
