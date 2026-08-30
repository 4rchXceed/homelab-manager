use rustls::{
    ClientConfig, RootCertStore, ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

use crate::{
    certgen::{CertGenerationError, generate_ca, generate_server_leaf},
    errors::HLMCommunicationError,
};

#[derive(Clone)]
pub struct SSLConfig {
    pub key_path: Option<String>,
    pub server_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
}

impl SSLConfig {
    pub fn new(key_path: Option<String>, cert_path: String) -> Self {
        return SSLConfig {
            key_path: key_path,
            server_cert_path: Some(cert_path),
            client_cert_path: None,
        };
    }

    pub fn new_client(cert_path: String) -> Self {
        return SSLConfig {
            key_path: None,
            server_cert_path: None,
            client_cert_path: Some(cert_path),
        };
    }

    pub fn new_server(key_path: String, cert_path: String) -> Self {
        return SSLConfig {
            key_path: Some(key_path),
            server_cert_path: Some(cert_path),
            client_cert_path: None,
        };
    }

    pub fn new_generated_server(
        server_cert_path: String,
        key_path: String,
        client_crt_path: String,
        bind_ips: Vec<String>,
    ) -> Result<Self, CertGenerationError> {
        let (ca_cert, ca_key) = generate_ca()?;
        let (server_cert, server_key) = generate_server_leaf(&bind_ips, &ca_cert, &ca_key)?;
        let cert_pem = server_cert
            .to_pem()
            .expect("Failed to convert server certificate to PEM");
        let ca_cert_pem = ca_cert
            .to_pem()
            .expect("Failed to convert CA certificate to PEM");
        let key_pem = server_key
            .private_key_to_pem_pkcs8()
            .expect("Failed to convert server private key to PEM");
        std::fs::write(&server_cert_path, cert_pem)
            .map_err(|_| CertGenerationError::FailedToWriteServerCertificate)?;
        std::fs::write(&key_path, key_pem)
            .map_err(|_| CertGenerationError::FailedToWriteServerKey)?;
        std::fs::write(&client_crt_path, ca_cert_pem)
            .map_err(|_| CertGenerationError::FailedToWriteClientCertificate)?;
        return Ok(SSLConfig {
            key_path: Some(key_path),
            server_cert_path: Some(server_cert_path),
            client_cert_path: Some(client_crt_path),
        });
    }

    pub(crate) fn generate_cert_store(
        &self,
        pem: String,
    ) -> Result<RootCertStore, HLMCommunicationError> {
        let mut root_cert_store = RootCertStore::empty();
        let cert = CertificateDer::from_pem_file(pem)
            .map_err(|_| HLMCommunicationError::CertFileReadError)?;
        root_cert_store
            .add(cert)
            .map_err(|e| HLMCommunicationError::CertFileNotAdded(e))?;
        return Ok(root_cert_store);
    }

    pub fn generate_client_config(&self) -> Result<ClientConfig, HLMCommunicationError> {
        if self.client_cert_path.is_none() {
            return Err(HLMCommunicationError::MissingClientCertificate);
        }
        let root_cert_store =
            self.generate_cert_store(self.client_cert_path.as_ref().unwrap().clone())?;
        let client_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        return Ok(client_config);
    }

    pub fn generate_server_config(&self) -> Result<ServerConfig, HLMCommunicationError> {
        if self.server_cert_path.is_none() || self.key_path.is_none() {
            return Err(HLMCommunicationError::MissingServerCertificate);
        }
        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![
                    CertificateDer::from_pem_file(self.server_cert_path.as_ref().unwrap())
                        .map_err(|_| HLMCommunicationError::CertFileReadError)?,
                ],
                PrivateKeyDer::from_pem_file(
                    self.key_path
                        .clone()
                        .ok_or(HLMCommunicationError::MissingKeyPath)?,
                )
                .map_err(|_| HLMCommunicationError::KeyFileReadError)?,
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
        let config = SSLConfig::new(None, String::from("some/invalid/path"));
        let result = config.generate_cert_store(String::from("some/invalid/path"));
        assert!(result.is_err());

        let pathbase = "/tmp/unittest/test_communication_config_getstore";

        ensure_pathbase(pathbase);

        let config = SSLConfig::new_generated_server(
            format!("{}/key1", pathbase),
            format!("{}/key2", pathbase),
            format!("{}/key3", pathbase),
            vec![String::from("192.168.1.1")],
        );
        let config = config.unwrap();

        let result = config.generate_cert_store(config.client_cert_path.as_ref().unwrap().clone());
        assert!(result.is_ok());
    }

    #[test]
    fn test_communication_config_getclientconfig() {
        let pathbase = "/tmp/unittest/test_communication_config_getclientconfig";

        ensure_pathbase(pathbase);

        let config = SSLConfig::new_generated_server(
            format!("{}/key1", pathbase),
            format!("{}/key2", pathbase),
            format!("{}/key3", pathbase),
            vec![String::from("192.168.1.1")],
        );
        let config = config.unwrap();

        let result = config.generate_client_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_communication_config_getserverconfig() {
        let pathbase = "/tmp/unittest/test_communication_config_getserverconfig";
        ensure_pathbase(pathbase);
        let config = SSLConfig::new_generated_server(
            format!("{}/key1", pathbase),
            format!("{}/key2", pathbase),
            format!("{}/key3", pathbase),
            vec![String::from("192.168.1.1")],
        );
        let config = config.unwrap();
        let result = config.generate_server_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_communication_config_client() {
        let pathbase = "/tmp/unittest/test_communication_config_client";
        ensure_pathbase(pathbase);
        let config_srv = SSLConfig::new_generated_server(
            format!("{}/key1", pathbase),
            format!("{}/key2", pathbase),
            format!("{}/key3", pathbase),
            vec![String::from("192.168.1.1")],
        );
        assert!(config_srv.is_ok());
        let config = SSLConfig::new_client(format!("{}/key1", pathbase));
        let client_config = config.generate_client_config();
        assert!(client_config.is_ok());
    }
}
