use clap::Parser;
use futures::{executor::block_on, FutureExt};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::process::exit;
use traffiq::{
    config::{Commands, Options},
    tcp::connect_to_tcp,
    tcp::connect_to_tcp_over_tls,
    tcp::connect_to_tcp_with_payload_execution,
    tcp::create_tcp_over_tls_server,
    tcp::create_tcp_server,
    tcp::create_tcp_server_with_payload_execution,
    udp::connect_to_udp,
    udp::create_udp_server,
    uds::connect_to_uds,
    uds::create_uds_server,
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
        Commands::Listen {
            bind_host,
            port,
            tls,
            cert,
            key,
            udp,
            exec,
            uds,
            uds_path,
        } => {
            let listen_future = if *tls {
                create_tcp_over_tls_server(
                    Some(bind_host.as_str()),
                    Some(*port),
                    cert.clone().expect("cert is required."),
                    key.clone().expect("key is required"),
                )
                .boxed()
            } else if *udp {
                create_udp_server(Some(bind_host.as_str()), Some(*port)).boxed()
            } else if let Some(exec) = exec {
                create_tcp_server_with_payload_execution(bind_host, port, exec.clone()).boxed()
            } else if *uds {
                create_uds_server(uds_path.clone().expect("uds-path is required.")).boxed()
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
