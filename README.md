# Traffiq

Traffiq is a simple utility which reads and writes data across network connections, using HTTP, TCP or UDP protocol. It is designed to be a reliable "back-end" tool that can be used directly or easily driven by other programs and scripts.

A netcat inspired implementation written in Rust. It implements the following features:

- [x] Connect to Remote Host
- [x] Create a tcp(s)/http(s)/udp/uds server
- [x] Handle multiple connections to the server
- [x] Send and receive data from the server
- [x] Connect to hosts over TLS
- [x] Reverse shell to execute code on remote hosts.

### Requirements

- Rust v1.75+

### Usage

```
traffiq listen [OPTIONS] --port <PORT> <BIND_HOST>
Arguments:
  <BIND_HOST>  The host to bind the listener to

Options:
  -p, --port <PORT>          The port to bind the listener to
      --tls                  Use TLS for the connection
      --cert <CERT>          The path to the certificate to use for TLS
      --key <KEY>            The path to the key to use for TLS
      --udp                  Use UDP for the connection
      --uds                  Spin up a UDS server (Unix only)
      --uds-path <UDS_PATH>  The path to the UDS socket (Unix only)
  -e, --exec <EXEC>          Execute a command on each incoming connection. (Use Caution!)
  -h, --help                 Print help
  -V, --version              Print version
```
or

```
traffiq connect [OPTIONS] --port <PORT> <HOST>
Arguments:
  <HOST>  The host to connect to

Options:
    -p, --port <PORT>                The port to connect to
        --tls                        Use TLS for the connection
        --uds                        Connect to a UDS socket (Unix only)
        --uds-path <UDS_PATH>        The path to the UDS socket (Unix only)
        --ca <CA>                    The path to the certificate to use for TLS
        --udp                        Connect using UDP
        --listen-port <LISTEN_PORT>  The port to listen on for UDP connections
    -e, --exec <EXEC>                Execute a command on the remote host upon connection. (Use Caution!)
    -h, --help                       Print help
    -V, --version                    Print version
```

---
Sriram

