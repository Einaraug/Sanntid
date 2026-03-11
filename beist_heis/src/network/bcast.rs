use crossbeam_channel as cbc;
use log::warn;
use serde::Deserialize;
use socket2::Socket;
use std::mem::MaybeUninit;
use std::error;
use std::str;

use super::sock;

const UDP_BUF_SIZE: usize = 4096;

/// Serializes values received from `tx_channel` to JSON and broadcasts them over UDP.
/// Runs forever — intended to be spawned as a dedicated thread.
pub fn broadcast_udp<T: serde::Serialize>(port: u16, tx_channel: cbc::Receiver<T>) -> std::io::Result<()> {
    let (socket, broadcast_addr) = sock::new_broadcast_socket(port)?;

    loop {
        let value     = tx_channel.recv().unwrap();
        let json      = serde_json::to_string(&value).unwrap();
        if let Err(e) = socket.send_to(json.as_bytes(), &broadcast_addr) {
            warn!("UDP send failed: {}", e);
        }
    }
}

/// Listens for incoming UDP packets, deserializes them from JSON, and forwards
/// them to `rx_channel`. Runs forever — intended to be spawned as a dedicated thread.
pub fn receive_udp<T: serde::de::DeserializeOwned>(port: u16, rx_channel: cbc::Sender<T>) -> std::io::Result<()> {
    let socket  = sock::new_receiver_socket(port)?;
    let mut buf: [MaybeUninit<u8>; UDP_BUF_SIZE] = [MaybeUninit::uninit(); UDP_BUF_SIZE];

    loop {
        match deserialize_packet(&socket, &mut buf) {
            Ok(value) => rx_channel.send(value).unwrap(),
            Err(e)    => warn!("UDP receive failed: {}", e),
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Reads one UDP packet from `socket` into `buf` and deserializes it into `T`.
/// Uses MaybeUninit buffer to avoid zeroing memory on every call — recv() fills
/// it before we read it, so the unsafe slice construction is sound.
fn deserialize_packet<'a, T: Deserialize<'a>>(
    socket: &'_ Socket,
    buf:    &'a mut [MaybeUninit<u8>; UDP_BUF_SIZE],
) -> Result<T, Box<dyn error::Error>> {
    let bytes_received = socket.recv(buf)?;
    let bytes          = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, bytes_received) };
    let json_str       = str::from_utf8(bytes)?;
    serde_json::from_str::<T>(json_str).map_err(|e| e.into())
}
