use std::io;
use std::net;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

// Creates the tx and rx sockets used for UDP broadcasting
// From https://github.com/TTK4145/network-rust/blob/master/src/udpnet/sock.rs //TODO: Keep this?

pub fn new_broadcast_tx(port: u16) -> io::Result<(Socket, SockAddr)> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_broadcast(true)?;
    sock.set_reuse_address(true)?;
    let remote_addr = net::SocketAddr::from(([255, 255, 255, 255], port));
    Ok((sock, remote_addr.into()))
}

pub fn new_rx(port: u16) -> io::Result<Socket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    sock.set_broadcast(true)?;
    sock.set_reuse_address(true)?;
    let local_addr = net::SocketAddr::from(([0, 0, 0, 0], port));
    sock.bind(&local_addr.into())?;
    Ok(sock)
}