use std::env;
use std::net::UdpSocket;
use std::process::Command;
use std::thread;
use std::time::Duration;

const PORT: u16 = 34567;
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(2);
const COUNT_INTERVAL: Duration = Duration::from_millis(500);

fn main() {
    let mut counter: u64 = 0;

    // === BACKUP PHASE ===
    // Bind to the well-known port and listen for heartbeats from the primary.
    // If we time out, the primary is dead and we take over.
    println!("[BACKUP] Listening for primary on port {}...", PORT);

    let socket = match UdpSocket::bind(("127.0.0.1", PORT)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not bind to port {}: {}. Another backup may already be running.", PORT, e);
            return;
        }
    };
    socket
        .set_read_timeout(Some(HEARTBEAT_TIMEOUT))
        .expect("Failed to set read timeout");

    let mut buf = [0u8; 64];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, _)) => {
                if let Ok(msg) = std::str::from_utf8(&buf[..size]) {
                    if let Ok(n) = msg.parse::<u64>() {
                        counter = n;
                    }
                }
            }
            Err(_) => {
                // Timeout: no heartbeat received — primary is dead
                println!(
                    "[BACKUP] No heartbeat received. Taking over at count {}.",
                    counter
                );
                break;
            }
        }
    }

    // Close the listening socket so the new backup can bind to the same port
    drop(socket);

    // === TRANSITION: spawn new backup ===
    spawn_backup();

    // Brief pause to let the backup bind to the port
    thread::sleep(Duration::from_millis(300));

    // === PRIMARY PHASE ===
    println!("[PRIMARY] Counting from {}...", counter + 1);

    let send_socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to create send socket");

    loop {
        counter += 1;
        println!("{}", counter);

        // Send heartbeat with the latest printed count
        let msg = counter.to_string();
        let _ = send_socket.send_to(msg.as_bytes(), ("127.0.0.1", PORT));

        thread::sleep(COUNT_INTERVAL);
    }
}

fn spawn_backup() {
    let exe = env::current_exe().expect("Failed to get current exe path");
    let exe_str = exe.to_str().expect("Invalid exe path");

    // macOS: open a new Terminal window running the backup
    let escaped = exe_str.replace("'", "'\\''");
    let script = format!(
        "tell app \"Terminal\" to do script \"'{}'\"",
        escaped
    );

    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .expect("Failed to spawn backup process");

    println!("[PRIMARY] Backup spawned.");
}
