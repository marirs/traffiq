use futures::{stream, StreamExt};
use log::{debug, info};
use std::{ops::RangeInclusive, sync::Arc, time::Duration};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

pub mod config;
mod error;
pub mod tcp;
pub mod udp;
mod utils;

pub mod http;
#[cfg(target_family = "unix")]
pub mod uds;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

/// Default Localhost
pub const LOCALHOST: &str = "127.0.0.1";

/// Default Any Host
pub const ANY_HOST: &str = "0.0.0.0";

/// Most Common Ports
pub const MOST_COMMON_PORTS_1002: &[u16] = &[
    5601, 9300, 80, 23, 443, 21, 22, 25, 3389, 110, 445, 139, 143, 53, 135, 3306, 8080, 1723, 111,
    995, 993, 5900, 1025, 587, 8888, 199, 1720, 465, 548, 113, 81, 6001, 10000, 514, 5060, 179,
    1026, 2000, 8443, 8000, 32768, 554, 26, 1433, 49152, 2001, 515, 8008, 49154, 1027, 5666, 646,
    5000, 5631, 631, 49153, 8081, 2049, 88, 79, 5800, 106, 2121, 1110, 49155, 6000, 513, 990, 5357,
    427, 49156, 543, 544, 5101, 144, 7, 389, 8009, 3128, 444, 9999, 5009, 7070, 5190, 3000, 5432,
    1900, 3986, 13, 1029, 9, 5051, 6646, 49157, 1028, 873, 1755, 2717, 4899, 9100, 119, 37, 1000,
    3001, 5001, 82, 10010, 1030, 9090, 2107, 1024, 2103, 6004, 1801, 5050, 19, 8031, 1041, 255,
    27017, 5432, 5050,
];

/// Max Port number
pub const MAX_PORT: usize = 65535;

/// Port Range
pub const PORT_RANGE: RangeInclusive<usize> = 1..=MAX_PORT;

/// Check for the open ports on a host.
pub async fn check_open_ports_v2(host: &str, ports: Vec<u16>) -> Result<()> {
    let open_ports = Arc::new(Mutex::new(Vec::new()));
    info!("Checking for open ports on {}", host);
    let host = host.to_string();
    let mut handles = vec![];
    for port in ports {
        let open_ports = Arc::clone(&open_ports);
        let host = host.clone();
        handles.push(tokio::spawn(async move {
            let host = host.clone();
            debug!("Checking port {} on {}", port, host);
            let addr = format!("{host}:{port}");
            if let Ok(mut stream) = TcpStream::connect(addr.as_str()).await {
                debug!("Port {port} is open on {host}");
                open_ports.lock().await.push(port);
                stream.shutdown().await.unwrap();
            } else {
                debug!("Port {} is closed on {}", port, host);
            }
        }));
    }
    futures::future::join_all(handles).await;
    if !open_ports.lock().await.is_empty() {
        info!("Open ports on \"{}\": {:?}", host, open_ports.lock().await);
    } else {
        info!("No open ports on {}", host);
    }
    Ok(())
}

pub async fn check_open_ports(host: &str, ports: Vec<u16>) -> Result<()> {
    info!("Checking for open ports on {}", host);
    let ports = Box::new(ports.into_iter());
    let ports = stream::iter(ports);
    let timeout = Duration::from_millis(100);
    ports
        .for_each_concurrent(None, |port| async move {
            debug!("Checking port {} on {}", port, host);
            let addr = format!("{host}:{port}");
            if let Ok(Ok(_)) =
                tokio::time::timeout(
                    timeout,
                    async move { TcpStream::connect(addr.as_str()).await },
                )
                .await
            {
                info!("Port {port} is open on {host}");
            }
        })
        .await;
    Ok(())
}
