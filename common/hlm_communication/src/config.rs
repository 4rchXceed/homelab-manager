use rustls::{
    ClientConfig, RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

use crate::{
    certgen::{CertGenerationError, generate_self_signed_cert},
    errors::HLMCommunicationError,
};

pub struct CommunicationConfig {
    pub key_path: Option<String>,
    pub cert_path: String,
    pub bind_addrs: Vec<String>,
}

impl CommunicationConfig {
    pub fn new(key_path: Option<String>, cert_path: String, bind_ips: Vec<String>) -> Self {
        return CommunicationConfig {
            key_path: key_path,
            cert_path: cert_path,
            bind_addrs: bind_ips,
        };
    }

    pub fn new_generated_server(
        cert_path: String,
        key_path: String,
        bind_ips: Vec<String>,
    ) -> Result<Self, CertGenerationError> {
        generate_self_signed_cert(&bind_ips, &cert_path, &key_path)?;
        return Ok(CommunicationConfig {
            key_path: Some(key_path),
            cert_path: cert_path,
            bind_addrs: bind_ips,
        });
    }

    pub(crate) fn generate_cert_store(&self) -> Result<RootCertStore, HLMCommunicationError> {
        let mut root_cert_store = RootCertStore::empty();
        let cert = CertificateDer::from_pem_file(self.cert_path.clone())
            .map_err(|e| HLMCommunicationError::CertFileReadError(e))?;
        root_cert_store
            .add(cert)
            .map_err(|e| HLMCommunicationError::CertFileNotAdded(e))?;
        return Ok(root_cert_store);
    }

    pub fn generate_client_config(&self) -> Result<ClientConfig, HLMCommunicationError> {
        let root_cert_store = self.generate_cert_store()?;
        let client_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        return Ok(client_config);
    }

    pub fn generate_server_config(&self) -> Result<ServerConfig, HLMCommunicationError> {
        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![
                    CertificateDer::from_pem_file(self.cert_path.clone())
                        .map_err(|e| HLMCommunicationError::CertFileReadError(e))?,
                ],
                PrivateKeyDer::from_pem_file(
                    self.key_path
                        .clone()
                        .ok_or(HLMCommunicationError::MissingKeyPath)?,
                )
                .map_err(|e| HLMCommunicationError::KeyFileReadError(e))?,
            )
            .map_err(|_| HLMCommunicationError::CouldNotConfigCert)?;
        return Ok(server_config);
    }
}

#[cfg(test)]
mod tests {
    use std::fs::create_dir_all;

    use super::*;

    fn ensure_pathbase(pathbase: &str) {
        if std::path::Path::new(pathbase).exists() {
            std::fs::remove_dir_all(pathbase).unwrap();
        }
        assert!(create_dir_all(pathbase).is_ok());
    }

    #[test]
    fn test_communication_config_getstore() {
        let config = CommunicationConfig::new(None, String::from("some/invalid/path"), Vec::new());
        let result = config.generate_cert_store();
        assert!(result.is_err());

        let pathbase = "/tmp/unittest/test_communication_config_getstore";

        ensure_pathbase(pathbase);

        let config = CommunicationConfig::new_generated_server(
            format!("{}/key1", pathbase),
            format!("{}/key2", pathbase),
            vec![String::from("192.168.1.1")],
        );
        let config = config.unwrap();

        let result = config.generate_cert_store();
        assert!(result.is_ok());
    }

    #[test]
    fn test_communication_config_getclientconfig() {
        let pathbase = "/tmp/unittest/test_communication_config_getclientconfig";

        ensure_pathbase(pathbase);

        let config = CommunicationConfig::new_generated_server(
            format!("{}/key1", pathbase),
            format!("{}/key2", pathbase),
            vec![String::from("192.168.1.1")],
        );
        let config = config.unwrap();

        let result = config.generate_client_config();
        assert!(result.is_ok());
    }
}
