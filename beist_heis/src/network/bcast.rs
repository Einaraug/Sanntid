use crossbeam_channel as cbc;
use log::warn;
use serde::Deserialize;
use socket2::Socket;
use std::mem::MaybeUninit;

use std::error;
use std::str;

use super::sock;

// Broadcast and receive udp-packages
// From https://github.com/TTK4145/network-rust/blob/master/src/udpnet/bcast.rs //TODO: keep this?


pub fn broadcast_udp<T: serde::Serialize>(port: u16, ch: cbc::Receiver<T>) -> std::io::Result<()> { 
    let (sock, sock_addr) = sock::new_broadcast_tx(port)?;

    loop {
        // Waits for a struct from the channel, serializes it to JSON, broadcasts it
        let data = ch.recv().unwrap();
        let serialized = serde_json::to_string(&data).unwrap();
        if let Err(e) = sock.send_to(serialized.as_bytes(), &sock_addr) {
            warn!("Unable to send packet, {}", e);
        }
    }
}


pub fn receive_udp<T: serde::de::DeserializeOwned>(port: u16, ch: cbc::Sender<T>) -> std::io::Result<()> {
    let sock = sock::new_rx(port)?;
    let mut buf: [MaybeUninit<u8>; 4096] = [MaybeUninit::uninit(); 4096];

    loop {
        // Waits for a UDP packet, tries to deserialize it, sends it to the channel
        match parse_packet(&sock, &mut buf) { 
            Ok(packet) => ch.send(packet).unwrap(), 
            Err(e) => warn!("Received bad package got error: {}", e),
        }
    }
}


// Helper fuction

// Receives one UDP packet and deserializes it into type T
// Buffer is uninitialized - recv() fills it before we read it
fn parse_packet<'a, T: Deserialize<'a>>(
    sock: &'_ Socket, 
    buf: &'a mut [MaybeUninit<u8>; 4096],
) -> Result<T, Box<dyn error::Error>> {
    let n = sock.recv(buf)?;
    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
    let msg = str::from_utf8(bytes)?;
    serde_json::from_str::<T>(msg).map_err(|e| e.into())
}