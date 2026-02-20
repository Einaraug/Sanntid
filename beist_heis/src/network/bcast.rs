use crossbeam_channel as cbc;
use log::warn;
use serde::Deserialize;
use socket2::Socket;
use std::mem::MaybeUninit;

use std::error;
use std::str;

use super::sock;


//Send function
pub fn udp_send<T: serde::Serialize>(port: u16, ch: cbc::Receiver<T>) -> std::io::Result<()> { //T is a placeholder for any type (WorldView:)
    let (s, addr) = sock::new_tx(port)?; //Creates the sending socket and broadcast address
    loop {
        let data = ch.recv().unwrap(); //Waits until something arrives on the channel. unwrap() means "crash if this fails"
        let serialized = serde_json::to_string(&data).unwrap(); //Converts the struct into a JSON string.
        if let Err(e) = s.send_to(serialized.as_bytes(), &addr) {
            warn!("Unable to send packet, {}", e);
        }
    }
}


//Receive function. Very similar to tx but mirrored:
pub fn udp_receive<T: serde::de::DeserializeOwned>(port: u16, ch: cbc::Sender<T>) -> std::io::Result<()> {
    let s = sock::new_rx(port)?;

    let mut buf: [MaybeUninit<u8>; 1024] = [MaybeUninit::uninit(); 1024];
    loop {
        match parse_packet(&s, &mut buf) { //waits for a UDP packet and tries to deserialize it. match handles two cases:
            Ok(d) => ch.send(d).unwrap(), // successfully got a packet, forward it onto the channel so your main code can receive it
            Err(e) => warn!("Received bad package got error: {}", e), //something went wrong (bad packet, not valid JSON etc), just log a warning and keep going
        }
    }
}


//Helper function
fn parse_packet<'a, T: Deserialize<'a>>(
    s: &'_ Socket,
    buf: &'a mut [MaybeUninit<u8>; 1024],
) -> Result<T, Box<dyn error::Error>> {
    let n = s.recv(buf)?;
    let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
    let msg = str::from_utf8(bytes)?;
    serde_json::from_str::<T>(msg).map_err(|e| e.into())
}

/*
tx — sits in a loop, waits for your code to give it a struct, serializes it to JSON, broadcasts it
rx — sits in a loop, waits for UDP packets, deserializes them back into a struct, forwards them to your code
Both use channels to talk to the rest of your program - your code never calls these functions directly after spawning them, it just uses the channel ends
*/