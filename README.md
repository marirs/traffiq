# Traffiq

[![Windows](https://github.com/marirs/traffiq/actions/workflows/windows.yml/badge.svg?branch=master)](https://github.com/marirs/traffiq/actions/workflows/windows.yml)
[![macOS](https://github.com/marirs/traffiq/actions/workflows/macos.yml/badge.svg?branch=master)](https://github.com/marirs/traffiq/actions/workflows/macos.yml)
[![Linux x86_64](https://github.com/marirs/traffiq/actions/workflows/linux_x86-64.yml/badge.svg?branch=master)](https://github.com/marirs/traffiq/actions/workflows/linux_x86-64.yml)
[![Linux Arm7](https://github.com/marirs/traffiq/actions/workflows/linux_arm7.yml/badge.svg?branch=master)](https://github.com/marirs/traffiq/actions/workflows/linux_arm7.yml)

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
./traffiq --help
A netcat equivalent in pure rust.

Usage: traffiq <COMMAND>

Commands:
  listen   Start a listener for incoming connections
  connect  Connect to the controlling host
  scan     Scan a host for open ports
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
or

```
./traffiq listen --help
Usage: traffiq listen [OPTIONS] --port <PORT> <BIND_HOST>

Arguments:
  <BIND_HOST>  The host to bind the listener to

Options:
  -p, --port <PORT>          The port to bind the listener to
      --tls                  Use TLS for the connection. Used with TCP and HTTP
      --cert <CERT>          The path to the certificate to use for TLS
      --key <KEY>            The path to the key to use for TLS
      --udp                  Use UDP for the connection
      --http                 Use a HTTP server for the connection
      --uds                  Use aa UDS server (Unix only) for the connection
      --uds-path <UDS_PATH>  The path to the UDS socket (Unix only)
  -e, --exec <EXEC>          Execute a command on each incoming connection. (Use Caution!)
  -h, --help                 Print help
  -V, --version              Print version
 
 ```

or
```
./traffiq connect --help
Usage: traffiq connect [OPTIONS] --port <PORT> <HOST>

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

### Examples
```bash
# Start a tcp server on port 9000
traffiq listen localhost --port 9000
```
```bash
# Connect to a tcp server on port 9000
traffiq connect --port 9000 localhost
```
```bash
# Start a tcp server on port 9000 and execute a command on each incoming connection
traffiq listen localhost --port 9000 --exec "echo 'Hello World'"
```
```bash
# Start a TLS server on port 9000
traffiq listen localhost --port 9000 --tls --cert /path/to/cert --key /path/to/key
```
```bash
# Start a UDP Server on port 8000
traffiq listen localhost --port 8000 --udp
```
```bash
Start a HTTP server on port 9000
traffiq listen localhost --port 9000 --http
```

---
Sriram

