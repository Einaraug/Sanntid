mod elevio;
mod elev_algo;
mod world_view;
mod peer_monitor;
mod network;
mod orders;
mod assigner;
mod counters;
mod node;
mod node_states;

use elevio::elev as hw;
use elevio::poll::{self, ButtonEvent};
use elev_algo::elevator::Elevator;
use elev_algo::fsm::{SensorEvent, CompletedOrder};
use elev_algo::elevator::{N_BUTTONS, N_FLOORS};
use world_view::WorldView;
use orders::OrderTable;
use crossbeam_channel as cbc;
use std::thread;
use std::time::Duration;

const WV_PORT: u16 = 20100;
const POLL_PERIOD: Duration = Duration::from_millis(25);
const ASSIGNER_PATH: &str = "./hall_request_assigner";

fn main() {
    let self_id: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Hardware init
    let hw_elev = hw::Elevator::init("localhost:15657", N_FLOORS as u8).unwrap();

    // ═══════════════════════════════════════════════════════════════
    // Channels
    // ═══════════════════════════════════════════════════════════════

    // hw_poll_buttons → node
    let (btn_tx, btn_rx) = cbc::unbounded::<ButtonEvent>();

    // hw_poll_sensors → fsm
    let (sensor_tx, sensor_rx) = cbc::unbounded::<SensorEvent>();

    // node → fsm (full request table)
    let (order_tx, order_rx) = cbc::unbounded::<[[bool; N_BUTTONS]; N_FLOORS]>();

    // fsm → node (elevator state)
    let (state_tx, state_rx) = cbc::unbounded::<Elevator>();

    // fsm → node (completed orders)
    let (completed_tx, completed_rx) = cbc::unbounded::<CompletedOrder>();

    // node → udp_tx
    let (to_net_tx, to_net_rx) = cbc::unbounded::<WorldView>();

    // udp_rx → node
    let (from_net_tx, from_net_rx) = cbc::unbounded::<WorldView>();

    // node → assigner (bounded(1): always sends latest snapshot, drops if busy)
    let (to_assigner_tx, to_assigner_rx) = cbc::bounded::<WorldView>(1);

    // assigner → node (assigned OrderTable)
    let (from_assigner_tx, from_assigner_rx) = cbc::unbounded::<OrderTable>();

    // ═══════════════════════════════════════════════════════════════
    // Thread 1: hw_poll_buttons → Node
    // ═══════════════════════════════════════════════════════════════
    let hw1 = hw_elev.clone();
    thread::spawn(move || poll::poll_buttons(hw1, btn_tx, POLL_PERIOD));

    // ═══════════════════════════════════════════════════════════════
    // Thread 2: hw_poll_sensors → FSM
    // ═══════════════════════════════════════════════════════════════
    let hw2 = hw_elev.clone();
    thread::spawn(move || poll::poll_sensors(hw2, sensor_tx, POLL_PERIOD));

    // ═══════════════════════════════════════════════════════════════
    // Thread 3: Node
    // ═══════════════════════════════════════════════════════════════
    let hw3 = hw_elev.clone();
    let wv = WorldView::new(self_id);
    thread::spawn(move || node::run(btn_rx, from_net_rx, state_rx, completed_rx, from_assigner_rx, wv, hw3, order_tx, to_net_tx, to_assigner_tx));

    // ═══════════════════════════════════════════════════════════════
    // Thread 4: FSM
    // ═══════════════════════════════════════════════════════════════
    let fsm = Elevator::new();
    thread::spawn(move || fsm.run(hw_elev, sensor_rx, order_rx, state_tx, completed_tx));

    // ═══════════════════════════════════════════════════════════════
    // Thread 5: Hall request assigner
    // ═══════════════════════════════════════════════════════════════
    thread::spawn(move || {
        for wv_snapshot in to_assigner_rx {
            match assigner::assign_hall_requests(&wv_snapshot, ASSIGNER_PATH) {
                Ok(order_table) => { let _ = from_assigner_tx.send(order_table); }
                Err(e) => eprintln!("assigner: {e}"),
            }
        }
    });

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

    // ═══════════════════════════════════════════════════════════════
    // Main thread: keep alive
    // ═══════════════════════════════════════════════════════════════
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
