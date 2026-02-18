//Use network folder instead




/*
// network.rs
// UDP broadcast network module.
//
// This module handles ONLY sending and receiving raw bytes over UDP broadcast.
// Serialization and deserialization of WorldView (or any other type) is
// intentionally left to the caller.
//
// Usage:
//
//   // Start the network. This binds sockets and spawns a listener thread.
//   let (net, rx) = Network::start().expect("failed to bind sockets");
//
//   // Send your serialized WorldView to all peers:
//   let bytes: Vec<u8> = serialize_my_world_view(&wv);
//   net.broadcast(&bytes);
//
//   // In your main loop (or a separate thread), receive messages from peers:
//   while let Ok(packet) = rx.recv() {
//       let wv = deserialize_my_world_view(&packet.data);
//       // ... handle it
//   }

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

// ─── Configuration ────────────────────────────────────────────────────────────

/// The broadcast address. Sends to all hosts on the local network.
const BROADCAST_ADDR: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 255);

/// Port that all nodes listen on for incoming broadcasts.
const LISTEN_PORT: u16 = 20212;

/// Port used for sending. Must differ from LISTEN_PORT so we can bind both
/// on the same machine during testing.
const SEND_PORT: u16 = 20211;

/// How long the receive thread waits for a packet before looping.
/// Keeps the thread responsive to shutdown without burning CPU.
const RECV_TIMEOUT_MS: u64 = 200;

/// Maximum UDP payload size we accept.
const MAX_PACKET_SIZE: usize = 4096;

// ─── Incoming packet ──────────────────────────────────────────────────────────

/// A raw packet received from a peer.
/// `data` is the raw bytes — deserialize however you need.
/// `from` is the sender's address, useful if you ever need to reply directly.
pub struct Packet {
    pub data: Vec<u8>,
    pub from: SocketAddrV4,
}

// ─── Network ──────────────────────────────────────────────────────────────────

pub struct Network {
    send_socket: UdpSocket,
    broadcast_target: SocketAddrV4,
}

impl Network {
    /// Bind sockets and start the background receive thread.
    ///
    /// Returns:
    ///   - `Network`          — use to call `broadcast()`
    ///   - `Receiver<Packet>` — poll this for incoming messages from peers
    ///
    /// The Receiver can be used blocking (`rx.recv()`) or non-blocking
    /// (`rx.try_recv()`). It delivers one `Packet` per UDP datagram received.
    pub fn start() -> std::io::Result<(Self, Receiver<Packet>)> {
        // ── Send socket ───────────────────────────────────────────────────────
        // Bind to SEND_PORT. We set broadcast = true so we can send to 255.255.255.255.
        let send_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, SEND_PORT))?;
        send_socket.set_broadcast(true)?;

        // ── Receive socket ────────────────────────────────────────────────────
        // Bind to LISTEN_PORT. Every peer binds to the same port, so broadcasts
        // are received by all of them simultaneously.
        let recv_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, LISTEN_PORT))?;
        recv_socket.set_broadcast(true)?;
        // Timeout so the receive loop can check for shutdown gracefully.
        recv_socket.set_read_timeout(Some(Duration::from_millis(RECV_TIMEOUT_MS)))?;

        // ── Channel ───────────────────────────────────────────────────────────
        // The background thread sends packets into `tx`.
        // The caller receives them from `rx`.
        let (tx, rx) = mpsc::channel::<Packet>();

        // ── Spawn receive thread ──────────────────────────────────────────────
        thread::spawn(move || receive_loop(recv_socket, tx));

        let net = Network {
            send_socket,
            broadcast_target: SocketAddrV4::new(BROADCAST_ADDR, LISTEN_PORT),
        };

        Ok((net, rx))
    }

    /// Broadcast raw bytes to all peers on the network.
    ///
    /// Call this with whatever bytes your serializer produces, e.g.:
    ///   net.broadcast(&my_serialized_world_view);
    pub fn broadcast(&self, data: impl AsRef<[u8]>) {
        if let Err(e) = self.send_socket.send_to(data.as_ref(), self.broadcast_target) {
            eprintln!("[network] broadcast error: {}", e);
        }
    }
}

// ─── Receive loop (runs in background thread) ─────────────────────────────────

fn receive_loop(socket: UdpSocket, tx: Sender<Packet>) {
    let mut buf = [0u8; MAX_PACKET_SIZE];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                // Parse the sender address. We only support IPv4.
                let from = match src {
                    std::net::SocketAddr::V4(addr) => addr,
                    _ => continue, // ignore IPv6, shouldn't happen on LAN
                };

                let packet = Packet {
                    data: buf[..len].to_vec(),
                    from,
                };

                // Forward to the caller. If the receiver has been dropped
                // (i.e. the rest of the program shut down), exit the thread.
                if tx.send(packet).is_err() {
                    break;
                }
            }

            // Timeout — normal, just loop again.
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue;
            }

            // Real error — log and keep trying.
            Err(e) => {
                eprintln!("[network] recv error: {}", e);
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Sends a packet from one socket and checks the receiver gets it.
    /// Run with: cargo test -- --nocapture
    #[test]
    fn test_send_and_receive() {
        let (net, rx) = Network::start().expect("failed to start network");

        let payload = b"hello elevator";
        net.broadcast(payload);

        // Give the packet time to loop back (localhost broadcast)
        let packet = rx.recv_timeout(Duration::from_secs(1))
            .expect("did not receive packet within 1 second");

        assert_eq!(&packet.data, payload);
    }
}


*/