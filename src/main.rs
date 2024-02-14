use clap::Parser;
use futures::{executor::block_on, FutureExt};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use std::net::ToSocketAddrs;
use std::process::exit;
#[cfg(target_family = "unix")]
use traffiq::uds::{connect_to_uds, create_uds_server};
use traffiq::{
    config::{Commands, Options},
    http::{create_http_server, HttpServerSettings, ServerConfig, SslConfig},
    tcp::{
        connect_to_tcp, connect_to_tcp_over_tls, connect_to_tcp_with_payload_execution,
        create_tcp_over_tls_server, create_tcp_server, create_tcp_server_with_payload_execution,
    },
    udp::{connect_to_udp, create_udp_server},
    Result,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_colors(true)
        .init()?;
    let options = Options::parse();
    match &options.command {
        #[cfg(target_family = "unix")]
        Commands::Listen {
            bind_host,
            port,
            tls,
            cert,
            key,
            http,
            udp,
            uds,
            uds_path,
            exec,
        } => {
            let listen_future = if *udp {
                create_udp_server(Some(bind_host.as_str()), Some(*port)).boxed()
            } else if let Some(exec) = exec {
                create_tcp_server_with_payload_execution(bind_host, port, exec.clone()).boxed()
            } else if *uds {
                create_uds_server(uds_path.clone().expect("uds-path is required.")).boxed()
            } else if *http {
                // make the bind_host resolve using DNS if required.
                let bind_host = format!("{}:{}", bind_host, port)
                    .to_socket_addrs()
                    .unwrap()
                    .last()
                    .unwrap()
                    .ip();
                let ssl_settings = if *tls {
                    if let Some(cert_file) = cert {
                        if let Some(key_file) = key {
                            Some(SslConfig {
                                enabled: true,
                                generate_self_signed: false,
                                key_file: key_file.clone(),
                                cert_file: cert_file.clone(),
                                ..Default::default()
                            })
                        } else {
                            // Missing Key file.
                            Some(SslConfig {
                                enabled: true,
                                ..Default::default()
                            })
                        }
                    } else {
                        Some(SslConfig {
                            enabled: true,
                            ..Default::default()
                        })
                    }
                } else {
                    None
                };
                let mut https_settings = HttpServerSettings {
                    server: ServerConfig {
                        host: bind_host,
                        port: *port as usize,
                        ..Default::default()
                    },
                    ssl: ssl_settings,
                };
                https_settings.init()?;
                create_http_server(https_settings).boxed()
            } else if *tls {
                let mut cert_info = None;
                if let Some(cert) = cert {
                    if let Some(key) = key {
                        cert_info = Some((cert.clone(), key.clone()));
                    }
                }
                info!("Creating TCP over TLS server");
                create_tcp_over_tls_server(Some(bind_host.as_str()), Some(*port), cert_info).boxed()
            } else {
                create_tcp_server(Some(bind_host.as_str()), Some(*port)).boxed()
            };
            block_on(async {
                tokio::select! {
                    res = listen_future => {
                        if let Err(e) = res {
                            eprintln!("Listening failed: {}", e);
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        exit(0);
                    }
                }
            });
        }
        #[cfg(not(target_family = "unix"))]
        Commands::Listen {
            bind_host,
            port,
            tls,
            cert,
            key,
            udp,
            exec,
            http,
        } => {
            let listen_future = if *udp {
                create_udp_server(Some(bind_host.as_str()), Some(*port)).boxed()
            } else if let Some(exec) = exec {
                create_tcp_server_with_payload_execution(bind_host, port, exec.clone()).boxed()
            } else if *http {
                // make the bind_host resolve using DNS if required.
                let bind_host = format!("{}:{}", bind_host, port)
                    .to_socket_addrs()
                    .unwrap()
                    .last()
                    .unwrap()
                    .ip();
                let ssl_settings = if *tls {
                    if let Some(cert_file) = cert {
                        if let Some(key_file) = key {
                            Some(SslConfig {
                                enabled: true,
                                generate_self_signed: false,
                                key_file: key_file.clone(),
                                cert_file: cert_file.clone(),
                                ..Default::default()
                            })
                        } else {
                            // Missing Key file.
                            Some(SslConfig {
                                enabled: true,
                                ..Default::default()
                            })
                        }
                    } else {
                        Some(SslConfig {
                            enabled: true,
                            ..Default::default()
                        })
                    }
                } else {
                    None
                };
                let mut https_settings = HttpServerSettings {
                    server: ServerConfig {
                        host: bind_host,
                        port: *port as usize,
                        ..Default::default()
                    },
                    ssl: ssl_settings,
                };
                https_settings.init()?;
                create_http_server(https_settings).boxed()
            } else if *tls {
                let mut cert_info = None;
                if let Some(cert) = cert {
                    if let Some(key) = key {
                        cert_info = Some((cert.clone(), key.clone()));
                    }
                }
                create_tcp_over_tls_server(Some(bind_host.as_str()), Some(*port), cert_info).boxed()
            } else {
                create_tcp_server(Some(bind_host.as_str()), Some(*port)).boxed()
            };
            block_on(async {
                tokio::select! {
                    res = listen_future => {
                        if let Err(e) = res {
                            eprintln!("Listening failed: {}", e);
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        exit(0);
                    }
                }
            });
        }
        #[cfg(target_family = "unix")]
        Commands::Connect {
            host,
            port,
            tls,
            ca,
            udp,
            listen_port,
            exec,
            uds,
            uds_path,
        } => {
            let connect_future = if *tls {
                connect_to_tcp_over_tls(host.as_str(), *port, ca).boxed()
            } else if *udp {
                connect_to_udp(
                    Some(host.as_str()),
                    *port,
                    (*listen_port).expect("listen-port is required."),
                )
                .boxed()
            } else if *uds {
                connect_to_uds(uds_path.clone().expect("uds-path is required.")).boxed()
            } else if let Some(exec) = exec {
                connect_to_tcp_with_payload_execution(host.as_str(), *port, exec.clone()).boxed()
            } else {
                connect_to_tcp(host.as_str(), *port).boxed()
            };
            block_on(async {
                tokio::select! {
                    res = connect_future => {
                        if let Err(e) = res {
                            eprintln!("Connecting failed: {}", e);
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        exit(0);
                    }
                }
            });
        }
        #[cfg(not(target_family = "unix"))]
        Commands::Connect {
            host,
            port,
            tls,
            ca,
            udp,
            listen_port,
            exec,
        } => {
            let connect_future = if *tls {
                connect_to_tcp_over_tls(host.as_str(), *port, ca).boxed()
            } else if *udp {
                connect_to_udp(
                    Some(host.as_str()),
                    *port,
                    (*listen_port).expect("listen-port is required."),
                )
                .boxed()
            } else if let Some(exec) = exec {
                connect_to_tcp_with_payload_execution(host.as_str(), *port, exec.clone()).boxed()
            } else {
                connect_to_tcp(host.as_str(), *port).boxed()
            };
            block_on(async {
                tokio::select! {
                    res = connect_future => {
                        if let Err(e) = res {
                            eprintln!("Connecting failed: {}", e);
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        exit(0);
                    }
                }
            });
        }
        Commands::Scan { host, lo, hi } => {
            let mut ports = traffiq::MOST_COMMON_PORTS_1002.to_vec();
            if let Some(lo) = lo {
                if let Some(hi) = hi {
                    ports = (*lo..=*hi).collect::<Vec<_>>();
                }
            }
            let scan_future = traffiq::check_open_ports(host.as_str(), ports).boxed();
            block_on(async {
                tokio::select! {
                    res = scan_future => {
                        if let Err(e) = res {
                            eprintln!("Scanning failed: {}", e);
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        exit(0);
                    }
                }
            });
        }
    }
    Ok(())
}
