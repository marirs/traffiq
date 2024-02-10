use crate::{
    error::Error::{DnsError, TlsError},
    utils::{read_write, read_write_exec},
    LOCALHOST,
};
use log::info;
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::{
    io::split,
    net::{TcpListener, TcpStream},
};
use tokio_rustls::{
    rustls::{
        Certificate, ClientConfig, OwnedTrustAnchor, PrivateKey, RootCertStore, ServerConfig,
    },
    TlsAcceptor, TlsConnector,
};

pub async fn connect_to_tcp_over_tls(
    host: &str,
    port: u16,
    ca: &Option<String>,
) -> crate::Result<()> {
    let mut root_store = RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject.to_vec(),
            ta.subject_public_key_info.to_vec(),
            ta.name_constraints.as_ref().map(|nc| nc.to_vec()),
        )
    }));
    if let Some(ca) = ca {
        for cert in load_certs(Path::new(ca.as_str()))? {
            root_store
                .add(&cert)
                .map_err(|_e| TlsError("Could not add CA.".to_string()))?;
        }
    }
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let rc_config = Arc::new(config);
    let tls_connector = TlsConnector::from(rc_config);
    let addr = format!("{host}:{port}");
    let server_name = host
        .try_into()
        .map_err(|_| DnsError("Invalid DNS Name.".to_string()))?;
    let socket = TcpStream::connect(addr).await?;
    let stream = tls_connector.connect(server_name, socket).await?;
    let (reader, writer) = split(stream);
    read_write(reader, writer).await?;
    Ok(())
}

pub async fn connect_to_tcp(host: &str, port: u16) -> crate::Result<()> {
    let addr = format!("{host}:{port}");
    let stream = TcpStream::connect(addr.as_str()).await?;
    info!("Connected to: {}...", stream.peer_addr()?);
    let (reader, writer) = stream.into_split();
    read_write(reader, writer).await?;
    Ok(())
}

/// Connect to a remote host with TCP and execute the given payload on the remote host.
pub async fn connect_to_tcp_with_payload_execution(
    host: &str,
    port: u16,
    payload: String,
) -> crate::Result<()> {
    let addr = format!("{host}:{port}");
    let stream = TcpStream::connect(addr.as_str()).await?;
    info!("Connected to: {}...", stream.peer_addr()?);
    let (reader, writer) = stream.into_split();
    read_write_exec(reader, writer, payload).await?;
    Ok(())
}

/// Create a TCP server listening on the given port. If none is given, use a random port.
pub async fn create_tcp_server(host: Option<&str>, port: Option<u16>) -> crate::Result<()> {
    let host = host.unwrap_or(LOCALHOST);
    let port = port.unwrap_or(0);
    let addr = format!("{host}:{port}");
    let tcp_listener = TcpListener::bind(addr.as_str()).await?;
    info!("TCP Server Listening on: {}...", tcp_listener.local_addr()?);
    loop {
        let (handle, _) = tcp_listener.accept().await?;
        let (reader, writer) = handle.into_split();
        read_write(reader, writer).await?;
    }
}

/// Create a server that serves TLS on the given port. If none is given, use a random port.
pub async fn create_tcp_over_tls_server(
    host: Option<&str>,
    port: Option<u16>,
    cert: String,
    key: String,
) -> crate::Result<()> {
    let host = host.unwrap_or(LOCALHOST);
    let port = port.unwrap_or(0);
    let addr = format!("{host}:{port}");
    let certs = load_certs(Path::new(cert.as_str()))?;
    let mut keys = load_keys(Path::new(key.as_str()))?;
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))?;
    let tls_acceptor = TlsAcceptor::from(Arc::new(config));
    let tcp_listener = TcpListener::bind(addr.as_str()).await?;
    info!("TLS Server Listening on: {}...", tcp_listener.local_addr()?);
    loop {
        let (handle, _) = tcp_listener.accept().await?;
        let stream = tls_acceptor.accept(handle).await?;
        let (reader, writer) = split(stream);
        read_write(reader, writer).await?;
    }
}

/// Load certificates from the given path.
fn load_certs(path: &Path) -> crate::Result<Vec<Certificate>> {
    let f = File::open(path)?;
    let certs = rustls_pemfile::certs(&mut BufReader::new(f))
        .map(|certs| Certificate(certs.unwrap().to_vec()))
        .collect::<Vec<_>>();
    info!(
        "Loaded {} certificates from \"{}\".",
        certs.len(),
        path.to_path_buf().display()
    );
    Ok(certs)
}

/// Load private keys from the given path.
fn load_keys(path: &Path) -> crate::Result<Vec<PrivateKey>> {
    let f = File::open(path)?;
    let keys = rustls_pemfile::rsa_private_keys(&mut BufReader::new(f))
        .map(|keys| PrivateKey(keys.unwrap().secret_pkcs1_der().to_vec()))
        .collect::<Vec<_>>();
    info!(
        "Loaded {} private keys from \"{}\".",
        keys.len(),
        path.to_path_buf().display()
    );
    Ok(keys)
}

/// Create a tcp server and execute the given payload on the each client that connects to this server.
pub async fn create_tcp_server_with_payload_execution(
    bind_host: &String,
    port: &u16,
    payload: String,
) -> crate::Result<()> {
    let addr = format!("{bind_host}:{port}");
    let listener = TcpListener::bind(addr.as_str()).await?;
    info!(
        "TCP Execution Server Listening on: {}...",
        listener.local_addr()?
    );
    loop {
        let (handle, _) = listener.accept().await?;
        let (reader, writer) = handle.into_split();
        read_write_exec(reader, writer, payload.clone()).await?;
    }
}
