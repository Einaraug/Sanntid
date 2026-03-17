mod elevio;
mod elev_algo;
mod world_view;
mod peer_monitor;
mod network;
mod orders;
mod assigner;
mod counters;
mod coordinator;
mod node_states;

use elevio::elev as hw;
use elevio::poll::{self, ButtonEvent};
use elev_algo::elevator::Elevator;
use elev_algo::fsm::{SensorEvent, CompletedOrder};
use elev_algo::elevator::{N_BUTTONS, N_FLOORS};
use world_view::{WorldView, N_NODES};
use orders::OrderTable;
use crossbeam_channel as cbc;
use std::thread;
use std::time::Duration;

const WV_PORT: u16 = 20100;
const POLL_PERIOD: Duration = Duration::from_millis(25);
const ASSIGNER_PATH: &str = "./hall_request_assigner";

fn parse_self_id() -> usize {
    match std::env::args().nth(1) {
        Some(raw) => {
            let id = raw.parse::<usize>().unwrap_or_else(|_| {
                eprintln!("Invalid node id '{raw}'. Usage: cargo run -- <node_id>");
                std::process::exit(2);
            });
            if id >= N_NODES {
                eprintln!("Node id {id} is out of range. Expected 0..{}", N_NODES - 1);
                std::process::exit(2);
            }
            id
        }
        None => 0,
    }
}

fn main() {
    let self_id = parse_self_id();
    let hw_elev = hw::Elevator::init("localhost:15657", N_FLOORS as u8).unwrap();

    // Channels
    // hw_poll_buttons → coordinator
    let (btn_tx, btn_rx) = cbc::unbounded::<ButtonEvent>();

    // hw_poll_sensors → fsm
    let (sensor_tx, sensor_rx) = cbc::unbounded::<SensorEvent>();

    // coordinator → fsm
    let (order_tx, order_rx) = cbc::unbounded::<[[bool; N_BUTTONS]; N_FLOORS]>();

    // fsm → coordinator
    let (state_tx, state_rx) = cbc::unbounded::<Elevator>();

    // fsm → coordinator
    let (completed_tx, completed_rx) = cbc::unbounded::<CompletedOrder>();

    // coordinator → udp_tx
    let (to_net_tx, to_net_rx) = cbc::unbounded::<WorldView>();

    // udp_rx → coordinator
    let (from_net_tx, from_net_rx) = cbc::unbounded::<WorldView>();

    // coordinator → assigner
    let (to_assigner_tx, to_assigner_rx) = cbc::bounded::<WorldView>(1);

    // assigner → coordinator
    let (from_assigner_tx, from_assigner_rx) = cbc::unbounded::<OrderTable>();

    // coordinator → lights
    let (to_lights_tx, to_lights_rx) = cbc::bounded::<OrderTable>(1);
   

    // Threads 
    let hw_buttons = hw_elev.clone();
    thread::spawn(move || poll::poll_buttons(hw_buttons, btn_tx, POLL_PERIOD));

    let hw_sensors = hw_elev.clone();
    thread::spawn(move || poll::poll_sensors(hw_sensors, sensor_tx, POLL_PERIOD));
    
    let hw_lights = hw_elev.clone();
    thread::spawn(move || elevio::lights::run(hw_lights, self_id, to_lights_rx));
    
    let wv = WorldView::new(self_id);
    thread::spawn(move || coordinator::run(btn_rx, from_net_rx, state_rx, completed_rx, from_assigner_rx, wv, order_tx, to_net_tx, to_assigner_tx, to_lights_tx));

    let fsm = Elevator::new();
    thread::spawn(move || fsm.run(hw_elev, sensor_rx, order_rx, state_tx, completed_tx));
    
    thread::spawn(move || assigner::run(to_assigner_rx, from_assigner_tx, ASSIGNER_PATH));

    // ═══════════════════════════════════════════════════════════════
    // Thread 6: UDP TX
    // ═══════════════════════════════════════════════════════════════
    thread::spawn(move || {
        network::bcast::broadcast_udp(WV_PORT, to_net_rx).unwrap();
    });

    // ═══════════════════════════════════════════════════════════════
    // Thread 7: UDP RX
    // ═══════════════════════════════════════════════════════════════
    thread::spawn(move || {
        network::bcast::receive_udp(WV_PORT, from_net_tx).unwrap();
    });

    // Keep the main thread alive (all work is done in spawned threads)
    loop { thread::sleep(Duration::MAX); }
}
