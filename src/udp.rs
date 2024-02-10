use crate::{error::Error::GenericError, ANY_HOST, LOCALHOST};
use log::info;
use tokio::{
    io::{stdin, stdout, AsyncReadExt, AsyncWriteExt},
    select,
};

/// Connect to a remote host with UDP.
pub async fn connect_to_udp(host: Option<&str>, port: u16, listen_port: u16) -> crate::Result<()> {
    let addr = format!("0.0.0.0:{listen_port}");
    let socket = tokio::net::UdpSocket::bind(addr.as_str()).await?;
    let remote_host = host.unwrap_or(LOCALHOST);
    let remote_addr = format!("{remote_host}:{port}");
    socket.connect(remote_addr.as_str()).await?;
    let mut stdin_buf = [0; 512];
    let mut network_in_buf = [0; 512];
    let mut stdin = stdin();
    let mut active = true;
    while active {
        select! {
            res = stdin.read(&mut stdin_buf) => {
                if let Ok(amount) = res {
                    if amount != 0 {
                        socket.send(&stdin_buf[0..amount]).await.map_err(|_| GenericError("Failed to write to the socket.".to_string()))?;
                    } else {
                        active = false;
                    }
                } else {
                    res.unwrap();
                }
            }
            res = socket.recv(&mut network_in_buf) => {
                if let Ok(amount) = res {
                    // Note there is no associated else because a zero length UDP message is valid. Just skip writing
                    if amount != 0 {
                        stdout().write(&network_in_buf[0..amount]).await.map_err(|_| GenericError("Failed to write to stdout.".to_string()))?;
                    }
                } else {
                    res.unwrap();
                }
            }
        }
    }
    Ok(())
}

/// Create a UDP server listening on the given port. If none is given, use a random port.
pub async fn create_udp_server(bind_host: Option<&str>, port: Option<u16>) -> crate::Result<()> {
    let host = bind_host.unwrap_or(ANY_HOST);
    let port = port.unwrap_or(0);
    let addr = format!("{host}:{port}");
    let socket = tokio::net::UdpSocket::bind(addr.as_str()).await?;
    info!("UDP Server Listening on: {}", socket.local_addr()?);
    let mut stdin_buf = [0; 512];
    let mut network_in_buf = [0; 512];
    let mut stdin = stdin();
    let mut is_connected = false;
    let mut stdin_tmp_buf = Vec::new();
    let mut active = true;
    while active {
        select!(
            res = socket.recv_from(&mut network_in_buf) => {
                if let Ok((amount, remote_addr)) = res {
                    // Note there is no associated else because a zero length UDP message is valid. Just skip writing
                    if amount != 0 {
                        stdout()
                        .write(&network_in_buf[0..amount])
                        .await
                        .map_err(|_| GenericError("Failed to write to stdout.".to_string()))?;
                    }
                    if !is_connected {
                        socket.connect(remote_addr).await.unwrap();
                        is_connected = true;

                        // Output pending data from stdin
                        if !stdin_tmp_buf.is_empty() {
                            socket
                            .send(stdin_tmp_buf.as_ref())
                            .await
                            .map_err(|_| GenericError("Failed to write to the socket.".to_string()))?;
                            stdin_tmp_buf.clear();
                        }
                    }
                } else {
                    res.unwrap();
                }
            }
            res = stdin.read(&mut stdin_buf) => {
                if let Ok(amount) = res {
                    if amount != 0 {
                        if is_connected {
                            socket.send(&stdin_buf[0..amount]).await.map_err(|_| GenericError("Failed to write to the socket.".to_string()))?;
                        } else {
                            stdin_tmp_buf.extend_from_slice(&stdin_buf[0..amount]);
                        }
                    } else {
                        active = false;
                    }
                } else {
                    res.unwrap();
                }
            }
        )
    }
    Ok(())
}
