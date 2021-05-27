use std::time::SystemTime;

use anyhow::{anyhow, Context};

#[test]
fn dummy_test() -> anyhow::Result<()> {
    let certificate = rcgen::generate_simple_self_signed(vec!["example.zengo.com".to_owned()])
        .context("generate self-signed certificate")?;
    let cert_pem = certificate.serialize_pem().context("serialize cert pem")?;
    let cert_der = certificate.serialize_der().context("serialize cert der")?;

    let mut root_certs = rustls::RootCertStore::empty();
    root_certs
        .add_pem_file(&mut cert_pem.as_bytes())
        .map_err(|_e| anyhow!("invalid serialized certificate"))?;
    let root_certs: Vec<_> = root_certs
        .roots
        .iter()
        .map(|c| c.to_trust_anchor())
        .collect();

    let now =
        webpki::Time::try_from(SystemTime::now()).map_err(|_| anyhow!("invalid system time"))?;

    let cert = webpki::EndEntityCert::from(&cert_der).context("construct EndEntityCert")?;
    cert.verify_is_valid_tls_server_cert(
        &[&webpki::ECDSA_P256_SHA256],
        &webpki::TLSServerTrustAnchors(&root_certs),
        &[],
        now,
    )
    .context("certificate is not valid")?;
    cert.verify_is_valid_for_dns_name(webpki::DNSNameRef::try_from_ascii_str("example.zengo.com")?)
        .context("certificate is not valid for its dns name")?;

    Ok(())
}
