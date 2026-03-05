#![allow(dead_code, unused_imports)]
mod elevio;
mod elev_algo;
mod world_view;
mod network;
mod orders;
mod assigner;
mod counters;

use elevio::elev as hw;
use elevio::poll::{self, ButtonEvent};
use elev_algo::elevator::Elevator;
use elev_algo::fsm::{SensorEvent, ConfirmedOrder};
use world_view::WorldView;
use crossbeam_channel as cbc;
use std::thread;
use std::time::Duration;

use crate::elev_algo::elevator::N_FLOORS;

const WV_PORT: u16 = 20100;
const POLL_PERIOD: Duration = Duration::from_millis(25);

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

    // hw_poll_buttons → worldview
    let (btn_tx, btn_rx) = cbc::unbounded::<ButtonEvent>();

    // hw_poll_sensors → fsm
    let (sensor_tx, sensor_rx) = cbc::unbounded::<SensorEvent>();

    // worldview → fsm (confirmed orders)
    let (order_tx, order_rx) = cbc::unbounded::<ConfirmedOrder>();

    // fsm → worldview (elevator state)
    let (state_tx, state_rx) = cbc::unbounded::<Elevator>();

    // worldview → udp_tx
    let (to_net_tx, to_net_rx) = cbc::unbounded::<WorldView>();

    // udp_rx → worldview
    let (from_net_tx, from_net_rx) = cbc::unbounded::<WorldView>();

    // ═══════════════════════════════════════════════════════════════
    // Thread 1: hw_poll_buttons → WorldView
    // ═══════════════════════════════════════════════════════════════
    let hw1 = hw_elev.clone();
    thread::spawn(move || poll::poll_buttons(hw1, btn_tx, POLL_PERIOD));

    // ═══════════════════════════════════════════════════════════════
    // Thread 2: hw_poll_sensors → FSM
    // ═══════════════════════════════════════════════════════════════
    let hw2 = hw_elev.clone();
    thread::spawn(move || poll::poll_sensors(hw2, sensor_tx, POLL_PERIOD));

    // ═══════════════════════════════════════════════════════════════
    // Thread 3: WorldView
    // ═══════════════════════════════════════════════════════════════
    let wv = WorldView::new(self_id);
    thread::spawn(move || wv.run(btn_rx, state_rx, from_net_rx, order_tx, to_net_tx));

    // ═══════════════════════════════════════════════════════════════
    // Thread 4: FSM
    // ═══════════════════════════════════════════════════════════════
    let fsm = Elevator::new();
    thread::spawn(move || fsm.run(hw_elev, sensor_rx, order_rx, state_tx));

    // ═══════════════════════════════════════════════════════════════
    // Thread 5: UDP TX
    // ═══════════════════════════════════════════════════════════════
    thread::spawn(move || {
        network::bcast::udp_send(WV_PORT, to_net_rx).unwrap();
    });

    // ═══════════════════════════════════════════════════════════════
    // Thread 6: UDP RX
    // ═══════════════════════════════════════════════════════════════
    thread::spawn(move || {
        network::bcast::udp_receive(WV_PORT, from_net_tx).unwrap();
    });

    // ═══════════════════════════════════════════════════════════════
    // Main thread: keep alive
    // ═══════════════════════════════════════════════════════════════
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
