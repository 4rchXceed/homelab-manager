use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::x509::extension::{
    AuthorityKeyIdentifier, BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName,
    SubjectKeyIdentifier,
};
use openssl::x509::{X509, X509NameBuilder};

use crate::common::net::errors::CertGenerationError;

/// ! Disclaimer: this has been "vibe coded", I don't know shit about OpenSSL / certs, so if you know how to do this code cleaner, please do so. I just wanted to get it working for now.
/// This is one of the rare times I use a 100% AI generated code. (maybe the only time in this project)
/// I belive AI transparency is an important concept, so I'll be honest about it. Thanks and sorry for the mess.
/// (Affected functions: generate_ca, generate_server_leaf)
pub(crate) fn generate_ca() -> Result<(X509, PKey<Private>), CertGenerationError> {
    let rsa = Rsa::generate(3072).map_err(|_| CertGenerationError::FailedToGenerateRSAKey)?;
    let ca_key = PKey::from_rsa(rsa).map_err(|_| CertGenerationError::FailedToGenerateRSAKey)?;

    let mut name_builder =
        X509NameBuilder::new().map_err(|_| CertGenerationError::FailedToBuildName)?;
    name_builder
        .append_entry_by_text("CN", "My Local CA")
        .map_err(|_| CertGenerationError::FailedToBuildName)?;
    let name = name_builder.build();

    let mut cert_builder =
        X509::builder().map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .set_version(2)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .set_subject_name(&name)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .set_issuer_name(&name)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .set_pubkey(&ca_key)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let not_before =
        Asn1Time::days_from_now(0).map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;
    let not_after = Asn1Time::days_from_now(3650)
        .map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;
    cert_builder
        .set_not_before(&not_before)
        .map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;
    cert_builder
        .set_not_after(&not_after)
        .map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;

    let mut serial = BigNum::new().map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;
    serial
        .rand(159, MsbOption::MAYBE_ZERO, false)
        .map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;
    let asn1_serial = serial
        .to_asn1_integer()
        .map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;
    cert_builder
        .set_serial_number(&asn1_serial)
        .map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;

    // CA = TRUE
    let basic_constraints = BasicConstraints::new()
        .critical()
        .ca()
        .build()
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .append_extension(basic_constraints)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let key_usage = KeyUsage::new()
        .critical()
        .key_cert_sign()
        .crl_sign()
        .build()
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .append_extension(key_usage)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    cert_builder
        .sign(&ca_key, MessageDigest::sha256())
        .map_err(|_| CertGenerationError::FailedToSignCertificate)?;
    Ok((cert_builder.build(), ca_key))
}

pub fn generate_server_leaf(
    ips: &[String],
    ca_cert: &X509,
    ca_key: &PKey<Private>,
) -> Result<(X509, PKey<Private>), CertGenerationError> {
    let rsa = Rsa::generate(2048).map_err(|_| CertGenerationError::FailedToGenerateRSAKey)?;
    let server_key =
        PKey::from_rsa(rsa).map_err(|_| CertGenerationError::FailedToGenerateRSAKey)?;

    let mut cert_builder =
        X509::builder().map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .set_version(2)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    if ips.is_empty() {
        return Err(CertGenerationError::FailedToAddValidIPAddresses);
    }

    let mut name_builder =
        X509NameBuilder::new().map_err(|_| CertGenerationError::FailedToBuildName)?;
    name_builder
        .append_entry_by_text("CN", &ips[0])
        .map_err(|_| CertGenerationError::FailedToBuildName)?;
    let name = name_builder.build();

    cert_builder
        .set_subject_name(&name)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    cert_builder
        .set_issuer_name(ca_cert.subject_name())
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .set_pubkey(&server_key)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let not_before =
        Asn1Time::days_from_now(0).map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;
    let not_after =
        Asn1Time::days_from_now(365).map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;
    cert_builder
        .set_not_before(&not_before)
        .map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;
    cert_builder
        .set_not_after(&not_after)
        .map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;

    let mut serial = BigNum::new().map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;
    serial
        .rand(159, MsbOption::MAYBE_ZERO, false)
        .map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;
    let asn1_serial = serial
        .to_asn1_integer()
        .map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;
    cert_builder
        .set_serial_number(&asn1_serial)
        .map_err(|_| CertGenerationError::FailedToSetSerialNumber)?;

    let basic_constraints = BasicConstraints::new()
        .critical()
        .build()
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .append_extension(basic_constraints)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let key_usage = KeyUsage::new()
        .critical()
        .digital_signature()
        .key_encipherment()
        .build()
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    cert_builder
        .append_extension(key_usage)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let ext_key_usage = ExtendedKeyUsage::new()
        .server_auth()
        .client_auth()
        .build()
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;
    cert_builder
        .append_extension(ext_key_usage)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let mut san_builder = SubjectAlternativeName::new();
    for ip in ips.iter() {
        if let Ok(ip) = ip.parse::<std::net::IpAddr>() {
            san_builder.ip(&ip.to_string());
        } else {
            san_builder.dns(ip);
        }
    }

    let san_ext = {
        let ctx = cert_builder.x509v3_context(Some(ca_cert), None);
        san_builder
            .build(&ctx)
            .map_err(|_| CertGenerationError::FailedToAddValidIPAddresses)?
    };
    cert_builder
        .append_extension(san_ext)
        .map_err(|_| CertGenerationError::FailedToAddValidIPAddresses)?;

    let ski = {
        let ctx = cert_builder.x509v3_context(Some(ca_cert), None);
        SubjectKeyIdentifier::new()
            .build(&ctx)
            .map_err(|_| CertGenerationError::FailedToBuildCertificate)?
    };
    cert_builder
        .append_extension(ski)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let aki = {
        let ctx = cert_builder.x509v3_context(Some(ca_cert), None);
        AuthorityKeyIdentifier::new()
            .keyid(false)
            .issuer(false)
            .build(&ctx)
            .map_err(|_| CertGenerationError::FailedToBuildCertificate)?
    };
    cert_builder
        .append_extension(aki)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    cert_builder
        .sign(ca_key, MessageDigest::sha256())
        .map_err(|_| CertGenerationError::FailedToSignCertificate)?;

    Ok((cert_builder.build(), server_key))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_generate_self_signed_cert() {
        let ips = vec![String::from("192.168.1.1")];
        let out_dir = std::path::Path::new("/tmp/cert_test");
        if out_dir.exists() {
            std::fs::remove_dir_all(out_dir).unwrap();
        }
        std::fs::create_dir_all(out_dir).unwrap();
        let (ca_cert, ca_key) = generate_ca().expect("Failed to generate CA");
        let r = generate_server_leaf(&ips, &ca_cert, &ca_key);
        assert!(r.is_ok());
        let (server_cert, server_key) = r.unwrap();
        let cert_pem = server_cert
            .to_pem()
            .expect("Failed to convert server certificate to PEM");
        let ca_cert_pem = ca_cert
            .to_pem()
            .expect("Failed to convert CA certificate to PEM");
        let key_pem = server_key
            .private_key_to_pem_pkcs8()
            .expect("Failed to convert server private key to PEM");
        std::fs::write(format!("{}/ca.crt", out_dir.display()), ca_cert_pem)
            .expect("Failed to write ca.crt");
        std::fs::write(format!("{}/server.key", out_dir.display()), key_pem)
            .expect("Failed to write server.key");
        std::fs::write(format!("{}/server.crt", out_dir.display()), cert_pem)
            .expect("Failed to write server.crt");
        if Path::new(&format!("{}/ca.crt", out_dir.display())).exists() {
            std::fs::remove_file(&format!("{}/ca.crt", out_dir.display())).unwrap();
        } else {
            panic!("ca.crt file was not created");
        }
        if Path::new(&format!("{}/server.key", out_dir.display())).exists() {
            std::fs::remove_file(&format!("{}/server.key", out_dir.display())).unwrap();
        } else {
            panic!("server.key file was not created");
        }
        std::fs::remove_dir_all(out_dir).unwrap();
    }
}
