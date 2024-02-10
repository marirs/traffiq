use crate::utils::read_write;
use log::info;
use tokio::io::split;

#[cfg(target_family = "unix")]
/// Create a unix Domain socket and listen on it.
pub async fn create_uds_server(path: String) -> crate::Result<()> {
    let listener = tokio::net::UnixListener::bind(path)?;
    info!("UDS Listening on: {:?}...", listener.local_addr()?);
    loop {
        let (stream, socket) = listener.accept().await?;
        info!("UDS accepted connection from {:?}..", socket);
        let (reader, writer) = split(stream);
        read_write(reader, writer).await?;
    }
}

#[cfg(target_family = "unix")]
/// Connect to a unix domain socket.
pub async fn connect_to_uds(path: String) -> crate::Result<()> {
    let stream = tokio::net::UnixStream::connect(path).await?;
    info!("UDS Connected to: {:?}...", stream.peer_addr()?);
    let (reader, writer) = stream.into_split();
    read_write(reader, writer).await?;
    Ok(())
}
