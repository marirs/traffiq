use openssl::{
    asn1::Asn1Time,
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{X509Name, X509},
};
use std::process::Stdio;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    process::Command,
    select,
};

#[derive(Debug)]
pub struct ResultCert {
    pub x509_certificate: X509,
    pub private_key: PKey<Private>,
}

/// Common Name
const CN: &str = "localhost";
/// Distinguished Name
const DN: &str = "localhost";
/// Subject Alternate Name
const SUB_ALT_NAME: &str = "local";
/// Country Code
const ISO_COUNTRY: &str = "US";
/// Organisation Name
const ORG_NAME: &str = "Organisation";
/// Issuer Name/Entity
const ISSUER: &str = "Organisation";
/// Issuer Alternate Name
const ISSUER_ALT: &str = "Organisation";
/// SSL Validity
const VALIDITY: u32 = 30; // 30 days

/// Stream from the reader and write to the writer.
pub async fn read_write<R, W>(mut reader: R, mut writer: W) -> crate::Result<()>
where
    R: AsyncRead + Unpin + Sized + Send + 'static,
    W: AsyncWrite + Unpin + Sized + Send + 'static,
{
    let client_read =
        tokio::spawn(async move { tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await });

    let client_write =
        tokio::spawn(async move { tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await });

    select! {
        _ = client_read => {}
        _ = client_write => {}
    }
    Ok(())
}

pub async fn read_write_exec<R, W>(mut reader: R, mut writer: W, exec: String) -> crate::Result<()>
where
    R: AsyncRead + Unpin + 'static,
    W: AsyncWrite + Unpin + 'static,
{
    let child = Command::new(exec)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.unwrap();
    let mut stdout = child.stdout.unwrap();
    let mut stderr = child.stderr.unwrap();

    let mut stdout_buf = [0; 512];
    let mut stderr_buf = [0; 512];
    let mut network_in_buf = [0; 512];

    let mut active = true;

    while active {
        select! {
            res = stdout.read(&mut stdout_buf) => {
                if let Ok(amount) = res {
                    if amount != 0 {
                        writer.write(&stdout_buf[0..amount]).await.map_err(|_| "failed to write to the socket").unwrap();
                    } else {
                        active = false;
                    }
                } else {
                    res.unwrap();
                }
            }
            res = stderr.read(&mut stderr_buf) => {
                if let Ok(amount) = res {
                    if amount != 0 {
                        writer.write(&stderr_buf[0..amount]).await.map_err(|_| "failed to write to the socket").unwrap();
                    } else {
                        active = false;
                    }
                } else {
                    res.unwrap();
                }
            }
            res = reader.read(&mut network_in_buf) => {
                if let Ok(amount) = res {
                    if amount != 0 {
                        stdin.write(&network_in_buf[0..amount]).await.map_err(|_| "failed to write to stdout").unwrap();
                    }
                } else {
                    res.unwrap();
                }
            }
        }
    }
    Ok(())
}

pub fn generate_cert() -> ResultCert {
    //! Generates a SSL Certificate
    //!
    //! ## Example usage:
    //!
    //! ```ignore
    //! let cert = generate_cert()
    //! ```
    let rsa = Rsa::generate(4096).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap();

    let mut name = X509Name::builder().unwrap();
    name.append_entry_by_nid(Nid::COMMONNAME, CN).unwrap();
    name.append_entry_by_nid(Nid::DISTINGUISHEDNAME, DN)
        .unwrap();
    name.append_entry_by_nid(Nid::SUBJECT_ALT_NAME, SUB_ALT_NAME)
        .unwrap();
    name.append_entry_by_nid(Nid::COUNTRYNAME, ISO_COUNTRY)
        .unwrap();
    name.append_entry_by_nid(Nid::ORGANIZATIONNAME, ORG_NAME)
        .unwrap();
    name.append_entry_by_nid(Nid::CERTIFICATE_ISSUER, ISSUER)
        .unwrap();
    name.append_entry_by_nid(Nid::ISSUER_ALT_NAME, ISSUER_ALT)
        .unwrap();
    let name = name.build();
    let time_before = Asn1Time::days_from_now(0).unwrap();
    let time_after = Asn1Time::days_from_now(VALIDITY).unwrap();
    let mut builder = X509::builder().unwrap();
    builder.set_version(1).unwrap();
    builder.set_subject_name(&name).unwrap();
    builder.set_issuer_name(&name).unwrap();
    builder.set_pubkey(&pkey).unwrap();
    builder.set_not_before(time_before.as_ref()).unwrap();
    builder.set_not_after(time_after.as_ref()).unwrap();
    builder.sign(&pkey, MessageDigest::sha256()).unwrap();

    let certificate: X509 = builder.build();
    ResultCert {
        x509_certificate: certificate,
        private_key: pkey,
    }
}
