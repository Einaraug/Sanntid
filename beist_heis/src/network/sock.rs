use std::io;
use std::net;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

/// Creates a UDP socket configured for broadcasting.
/// Returns the socket and the broadcast destination address (255.255.255.255:port).
pub fn new_broadcast_socket(port: u16) -> io::Result<(Socket, SockAddr)> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_broadcast(true)?;
    socket.set_reuse_address(true)?;
    let broadcast_addr = net::SocketAddr::from(([255, 255, 255, 255], port));
    Ok((socket, broadcast_addr.into()))
}

/// Creates a UDP socket bound to 0.0.0.0:port, ready to receive broadcast packets.
pub fn new_receiver_socket(port: u16) -> io::Result<Socket> {
    let socket     = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_broadcast(true)?;
    socket.set_reuse_address(true)?;
    let listen_addr = net::SocketAddr::from(([0, 0, 0, 0], port));
    socket.bind(&listen_addr.into())?;
    Ok(socket)
}
