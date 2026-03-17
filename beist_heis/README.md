# beist_heis

Fault-tolerant distributed elevator system in Rust, built for TTK4145 at NTNU.

## Overview
Up to N elevators coordinate over a local network. Each node broadcasts its full world-view (orders, elevator states, peer availability) via UDP. Conflicts are resolved using monotonic version counters (last-write-wins CRDT), so no central coordinator is needed.

Hall orders go through a confirm cycle — an order is only acted on once all available peers have acknowledged it. Assignment to a specific elevator is done by an external cost-function binary (`hall_request_assigner`).

## Architecture

```
poll_buttons  ──► coordinator ──► assigner ──► coordinator
poll_sensors  ──►     │                            │
                      ▼                            ▼
                     FSM ◄────────────── order table (assigned)
                      │
              UDP TX/RX (WorldView broadcast)
```

Each component runs in its own thread, communicating via crossbeam channels.

## Running

Start the elevator server (simulator or hardware), then:

```bash
cargo run -- <node_id>   # node_id: 0, 1, ... N_NODES
```

## Dependencies

- `hall_request_assigner` — precompiled binary in the project root
- `packet_loss` — network impairment scripts for testing in `Project-resources/packet_loss/`
