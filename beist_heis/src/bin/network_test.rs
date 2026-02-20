use crossbeam_channel as cbc;
use serde::{Deserialize, Serialize}; //to serialize and deserialise to JSON. Sending as JSON because its readable for humans and easier to debug. Could use bincode which serializes to compact binary instead of text.
use std::{env, thread, time::Duration}; //env lets us read command line arguments. thread lets us spawn background threads. Duration is for expressing time.

use beist_heis::network::bcast;


//Simple WorldView struct to use as test
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorldView {
    sender_id: u32,
    counter: u64,
}


//Main function
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect(); //Reads command line arguments into a list of strings.

    //If fewer than 3 arguments were given, print a usage message and exit with error code 1.
    if args.len() < 3 { 
        eprintln!("Bruk:\n  {} recv <port>\n  {} send <id> <port>", args[0], args[0]); 
        std::process::exit(1); //exit(0) all ok, exit(1) something is wrong
    }

    match args[1].as_str() { //Look at index 1 as a string and use as a switch for sending or receiving
        "recv" => {
            let port: u16 = args[2].parse().unwrap(); //Parse the port number from the third argument. .parse() converts a string to a number. .unwrap() crashes if it's not a valid number.
            let (tx, rx) = cbc::unbounded::<WorldView>(); //Creates a channel that can carry WorldView values. 
            
            //Spawn a background thread that runs bcast::rx
            thread::spawn(move || { //move means the thread takes ownership of port and tx
                let _ = bcast::udp_receive(port, tx); //let _ = means we're ignoring the return value.
            });

            loop {
                let wv = rx.recv().unwrap(); //rx.recv() blocks until a WorldView arrives from the background thread.
                println!("RECV: {:?}", wv); //prints WorldView using {:?} which uses the Debug derive to format it nicely.
            }
        }
        "send" => {
            let id: u32 = args[2].parse().unwrap();
            let port: u16 = args[3].parse().unwrap();
            let (tx, rx) = cbc::unbounded::<WorldView>();

            thread::spawn(move || {
                let _ = bcast::udp_send(port, rx);
            });
            

            // Each loop iteration increments the counter and sends a new WorldView onto the channel. 
            // The background bcast::tx thread picks it up and broadcasts it. 
            let mut counter = 0u64;
            loop {
                counter += 1;
                tx.send(WorldView {
                    sender_id: id,
                    counter,
                })
                .unwrap();

                println!("SEND {}", counter);
                thread::sleep(Duration::from_millis(200));
            }
        }
        _ => {
            eprintln!("Ukjent modus (bruk send/recv)");
            std::process::exit(1);
        }
    }
}