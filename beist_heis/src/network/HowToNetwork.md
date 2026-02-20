# Network Module — How to Use

## Overview
The network module handles sending and receiving data between elevator nodes over UDP broadcast.
It consists of two files:
- `sock.rs` — creates sockets (you never call this directly)
- `bcast.rs` — sends and receives any struct over the network

---

## Setup in main.rs

```rust
mod network {
    pub mod sock;
    pub mod bcast;
}

use crossbeam_channel as cbc;
use network::bcast;
```

---

## Sending a WorldView

```rust
// 1. Create a channel
let (tx, rx) = cbc::unbounded::<WorldView>();

// 2. Spawn the sender thread (do this once at startup)
thread::spawn(move || {
    bcast::tx(20001, rx).unwrap();
});

// 3. Send your WorldView whenever it changes
tx.send(my_world_view).unwrap();
```

---

## Receiving a WorldView

```rust
// 1. Create a channel
let (tx, rx) = cbc::unbounded::<WorldView>();

// 2. Spawn the receiver thread (do this once at startup)
thread::spawn(move || {
    bcast::rx(20001, tx).unwrap();
});

// 3. Receive WorldViews from peers in a loop
loop {
    let wv = rx.recv().unwrap();
    // handle incoming worldview from a peer
}
```

---

## WorldView must derive Serialize and Deserialize

```rust
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct WorldView {
    pub self_id: u32,
    // ... your fields
}
```

Every type inside WorldView needs this too (HallOrder, ElevatorDir etc).

---

## Cargo.toml dependencies

```toml
[dependencies]
crossbeam-channel = "0.5"
log = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
socket2 = "0.4"
```

---

## Full startup example

```rust
fn main() {
    let port = 20001;

    // outgoing channel — your code sends WorldViews into this
    let (wv_out_tx, wv_out_rx) = cbc::unbounded::<WorldView>();

    // incoming channel — your code receives WorldViews from this
    let (wv_in_tx, wv_in_rx) = cbc::unbounded::<WorldView>();

    // start network threads
    thread::spawn(move || bcast::udp_send(port, wv_out_rx).unwrap());
    thread::spawn(move || bcast::udp_receive(port, wv_in_tx).unwrap());

    // broadcast your worldview periodically
    let ticker = crossbeam_channel::tick(Duration::from_millis(50));
    thread::spawn(move || {
        loop {
            ticker.recv().unwrap();
            wv_out_tx.send(my_world_view.clone()).unwrap();
        }
    });

    // handle incoming worldviews from peers
    loop {
        let wv = wv_in_rx.recv().unwrap();
        // update last_seen timestamp for wv.self_id
        // merge wv into your local worldview
    }
}
```