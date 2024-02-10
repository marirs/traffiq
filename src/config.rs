use crate::{
    error::Error::{GenericError, InvalidPortError},
    Result,
};
use clap::{Parser, Subcommand};
use std::{ops::RangeInclusive, path::Path};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Options {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start a listener for incoming connections
    #[command(alias = "l")]
    Listen {
        /// The host to bind the listener to.
        bind_host: String,

        /// The port to bind the listener to.
        #[arg(long, short, value_parser = port_in_range)]
        port: u16,

        /// Use TLS for the connection.
        #[arg(long, default_value_t = false)]
        tls: bool,

        /// The path to the certificate to use for TLS.
        #[arg(long, value_parser = valid_path)]
        cert: Option<String>,

        /// The path to the key to use for TLS.
        #[arg(long, value_parser = valid_path)]
        key: Option<String>,

        /// Use UDP for the connection.
        #[arg(long, default_value_t = false)]
        udp: bool,

        #[cfg(target_family = "unix")]
        /// Spin up a UDS server (Unix only).
        #[arg(long, default_value_t = false)]
        uds: bool,

        /// The path to the UDS socket (Unix only).
        #[cfg(target_family = "unix")]
        #[arg(long, value_parser = valid_path)]
        uds_path: Option<String>,

        /// Execute a command on each incoming connection. (Use Caution!).
        #[arg(short, long)]
        exec: Option<String>,
    },

    /// Connect to the controlling host
    #[command(alias = "c")]
    Connect {
        /// The host to connect to.
        host: String,

        /// The port to connect to.
        #[arg(long, short, value_parser = port_in_range)]
        port: u16,

        /// Use TLS for the connection.
        #[arg(long, default_value_t = false)]
        tls: bool,

        /// Connect to a UDS socket (Unix only).
        #[cfg(target_family = "unix")]
        #[arg(long, default_value_t = false)]
        uds: bool,

        /// The path to the UDS socket (Unix only).
        #[cfg(target_family = "unix")]
        #[arg(long, value_parser = valid_path)]
        uds_path: Option<String>,

        /// The path to the certificate to use for TLS.
        #[arg(long, value_parser = valid_path)]
        ca: Option<String>,

        /// Connect using UDP.
        #[arg(long, default_value_t = false)]
        udp: bool,

        /// The port to listen on for UDP connections.
        #[arg(long, value_parser = port_in_range)]
        listen_port: Option<u16>,

        /// Execute a command on the remote host upon connection. (Use Caution!).
        #[arg(short, long)]
        exec: Option<String>,
    },

    /// Scan a host for open ports
    #[command(alias = "s")]
    Scan {
        /// The host to scan.
        host: String,

        /// The lower port to scan.
        #[arg(long, value_parser = port_in_range)]
        lo: Option<u16>,

        /// The upper port to scan.
        #[arg(long, value_parser = port_in_range)]
        hi: Option<u16>,
    },
}

const MAX_PORT: usize = 65535;
const PORT_RANGE: RangeInclusive<usize> = 1..=MAX_PORT;

fn port_in_range(port_str: &str) -> Result<u16> {
    let port: usize = port_str
        .parse()
        .map_err(|_| InvalidPortError(format!("{port_str} is not a valid port number.")))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(InvalidPortError(format!(
            "Port not in range {}-{}.",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        )))
    }
}

fn valid_path(s: &str) -> Result<String> {
    let path = Path::new(s);

    if path.exists() {
        Ok(s.to_string())
    } else {
        Err(GenericError(format!("Path does not exist {s}.")))
    }
}
