use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::extension::SubjectAlternativeName;
use openssl::x509::{X509, X509NameBuilder};
use std::fs;

#[derive(Debug)]
pub enum CertGenerationError {
    NoIPAddressesProvided,
    FailedToGenerateRSAKey,
    FailedToBuildName,
    FailedToBuildCertificate,
    FailedToSetValidityPeriod,
    FailedToSetSerialNumber,
    FailedToAddValidIPAddresses,
    FailedToSignCertificate,
    FailedToGeneratePrivateKey,
    FailedToWritePrivateKey,
    FailedToGenerateCertificate,
    FailedToWriteCertificate,
}

pub(crate) fn generate_self_signed_cert(
    ips: &Vec<String>,
    cert_file: &String,
    key_file: &String,
) -> Result<(), CertGenerationError> {
    if ips.is_empty() {
        return Err(CertGenerationError::NoIPAddressesProvided);
    }
    let first_ip = &ips[0];
    let rsa = Rsa::generate(3072).map_err(|_| CertGenerationError::FailedToGenerateRSAKey)?;
    let pkey = PKey::from_rsa(rsa).map_err(|_| CertGenerationError::FailedToGenerateRSAKey)?;

    let mut name_builder =
        X509NameBuilder::new().map_err(|_| CertGenerationError::FailedToBuildName)?;
    name_builder
        .append_entry_by_text("CN", first_ip)
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
        .set_pubkey(&pkey)
        .map_err(|_| CertGenerationError::FailedToBuildCertificate)?;

    let not_before =
        Asn1Time::days_from_now(0).map_err(|_| CertGenerationError::FailedToSetValidityPeriod)?;
    let not_after = Asn1Time::days_from_now(10000)
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

    let mut san_builder = SubjectAlternativeName::new();
    for ip in ips.iter() {
        if let Ok(ip) = ip.parse::<std::net::IpAddr>() {
            san_builder.ip(&ip.to_string());
        } else {
            san_builder.dns(ip);
        }
    }
    let san_ext = san_builder
        .build(&cert_builder.x509v3_context(None, None))
        .map_err(|_| CertGenerationError::FailedToAddValidIPAddresses)?;
    cert_builder
        .append_extension(san_ext)
        .map_err(|_| CertGenerationError::FailedToAddValidIPAddresses)?;

    cert_builder
        .sign(&pkey, MessageDigest::sha256())
        .map_err(|_| CertGenerationError::FailedToSignCertificate)?;
    let cert = cert_builder.build();

    let private_key = pkey
        .private_key_to_pem_pkcs8()
        .map_err(|_| CertGenerationError::FailedToGeneratePrivateKey)?;
    let cert_pem = cert
        .to_pem()
        .map_err(|_| CertGenerationError::FailedToGenerateCertificate)?;
    fs::write(key_file, private_key).map_err(|_| CertGenerationError::FailedToWritePrivateKey)?;
    fs::write(cert_file, cert_pem).map_err(|_| CertGenerationError::FailedToWriteCertificate)?;

    return Ok(());
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
        let r = generate_self_signed_cert(
            &ips,
            &format!("{}/cert.pem", out_dir.display()),
            &format!("{}/key.pem", out_dir.display()),
        );
        assert!(r.is_ok());
        if Path::new(&format!("{}/cert.pem", out_dir.display())).exists() {
            std::fs::remove_file(&format!("{}/cert.pem", out_dir.display())).unwrap();
        } else {
            panic!("cert.pem file was not created");
        }
        if Path::new(&format!("{}/key.pem", out_dir.display())).exists() {
            std::fs::remove_file(&format!("{}/key.pem", out_dir.display())).unwrap();
        } else {
            panic!("key.pem file was not created");
        }
        std::fs::remove_dir_all(out_dir).unwrap();
    }
}
