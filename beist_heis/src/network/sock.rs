use std::io;
use std::net;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};


//Create new sending socket
pub fn new_tx(port: u16) -> io::Result<(Socket, SockAddr)> {
    let sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    sock.set_broadcast(true)?;
    sock.set_reuse_address(true)?; //So all 3 nodes can share the same port. Without only one node could run per machine
    let remote_addr = net::SocketAddr::from(([255, 255, 255, 255], port));
    Ok((sock, remote_addr.into()))
}


//Create new receiving socket
pub fn new_rx(port: u16) -> io::Result<Socket> {
    let sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    sock.set_broadcast(true)?;
    sock.set_reuse_address(true)?;
    let local_addr = net::SocketAddr::from(([0, 0, 0, 0], port));
    sock.bind(&local_addr.into())?;
    Ok(sock)
}