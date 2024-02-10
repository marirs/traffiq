#![allow(clippy::enum_variant_names)]
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("TLS error: {0}")]
    TlsError(String),
    #[error("DNS error: {0}")]
    DnsError(String),
    #[error("Error: {0}")]
    GenericError(String),
    #[error("Invalid port: {0}")]
    InvalidPortError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Rustls error: {0}")]
    RustlsError(#[from] tokio_rustls::rustls::Error),
    #[error("Logging error: {0}")]
    LoggingError(#[from] log::SetLoggerError),
}
