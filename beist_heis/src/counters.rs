use serde::{Serialize, Deserialize};
use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::world_view::{WorldView, N_NODES, N_DIRS};

// Describes which part of the WorldView that was mutated by an operation
pub enum Change{
    Elevator {node_id: usize},
    PeerAvailability {node_id: usize},
    HallOrder {floor: usize, btn: Button},
    CabOrder {floor: usize, node_id: usize},
}

// Version counters counters implementing a last-write-wins CRDT (Conflict-free Replicated Data Type)
// Each field is a monotonically increasing version number that tracks changes in the WorldView
// System is required to restart before 64-bit counter is filled
// When merging WorldViews, the data with the highest corresponding counter overwrites the older data 
#[derive(Clone, Serialize, Deserialize)]
pub struct Counters {
    elevator: [u64; N_NODES],
    peer_availability: [u64; N_NODES],
    hall_order: [[u64; N_DIRS]; N_FLOORS],
    cab_order: [[u64; N_NODES]; N_FLOORS],
}

impl Counters {
    pub fn new() -> Self {
        Self {
            elevator: [0; N_NODES],
            peer_availability: [0; N_NODES],
            hall_order: [[0; N_DIRS]; N_FLOORS],
            cab_order: [[0; N_NODES]; N_FLOORS],
        }
    }

    // Incrementers
    pub fn inc_elevator(&mut self, node_id: usize) {
        self.elevator[node_id] += 1;
    }
    pub fn inc_peer_availability(&mut self, node_id: usize) {
        self.peer_availability[node_id] += 1;
    }
    pub fn inc_hall_order(&mut self, floor: usize, button: Button) {
        self.hall_order[floor][button.to_index()] += 1;
    }
    pub fn inc_cab_order(&mut self, floor: usize, node_id: usize) {
        self.cab_order[floor][node_id] += 1;
    }

    // Getters
    pub fn get_elevator(&self, node_id: usize) -> u64 {
        self.elevator[node_id]
    }
    pub fn get_peer_availability(&self, node_id: usize) -> u64 {
        self.peer_availability[node_id]
    }
    pub fn get_hall_order(&self, floor: usize, button: Button) -> u64 {
        self.hall_order[floor][button.to_index()]
    }
    pub fn get_cab_order(&self, floor: usize, node_id: usize) -> u64 {
        self.cab_order[floor][node_id]
    }

    // Setters
    pub fn set_elevator(&mut self, node_id: usize, value: u64) {
        self.elevator[node_id] = value;
    }
    pub fn set_peer_availability(&mut self, node_id: usize, value: u64) {
        self.peer_availability[node_id] = value;
    }
    pub fn set_hall_order(&mut self, floor: usize, button: Button, value: u64) {
        self.hall_order[floor][button.to_index()] = value;
    }
    pub fn set_cab_order(&mut self, floor: usize, node_id: usize, value: u64) {
        self.cab_order[floor][node_id] = value;
    }

    pub fn apply(&mut self, changes: Vec<Change>) {
        for change in changes {
            match change {
                Change::HallOrder{floor, btn}  => self.inc_hall_order(floor, btn),
                Change::CabOrder{floor, node_id } => self.inc_cab_order(floor, node_id),
                Change::Elevator{node_id} => self.inc_elevator(node_id),
                Change::PeerAvailability{node_id } => self.inc_peer_availability(node_id),
            }
        }
    }
}

// Merge protocol
pub fn merge(local: &mut WorldView, incoming: &WorldView) {
    merge_hall_orders(local, incoming);
    merge_cab_orders(local, incoming);
    merge_peer_availability(local, incoming);
    merge_elevators(local, incoming);
}

// Private helper functions for merge 
fn merge_hall_orders(local: &mut WorldView, incoming: &WorldView) {
    for floor in 0..N_FLOORS {
        for btn in [Button::HallUp, Button::HallDown] {
            let local_ct= local.counters.get_hall_order(floor, btn);
            let incoming_ct= incoming.counters.get_hall_order(floor, btn);

            if incoming_ct > local_ct {
                // Copy incoming into local
                let hall_order = incoming.order_table.get_hall_order(floor, btn.to_index());

                local.order_table.set_hall_order(floor, btn, hall_order);
                local.order_table.set_seen_by_hall(floor, btn, local.self_id);
                local.counters.set_hall_order(floor, btn, incoming_ct);
            } 
            else if incoming_ct == local_ct {
                // WorldViews are on the same version - confirm that the order has been seen by incoming
                local.order_table.set_seen_by_hall(floor, btn, incoming.self_id);
            }
        }
    }
}

// Same merge logic is implemented for CabOrders
fn merge_cab_orders(local: &mut WorldView, incoming: &WorldView) {
    for floor in 0..N_FLOORS {
        for node_id in 0..N_NODES {
            let local_ct    = local.counters.get_cab_order(floor, node_id);
            let incoming_ct = incoming.counters.get_cab_order(floor, node_id);

            if incoming_ct > local_ct {
                let cab_order = incoming.order_table.get_cab_order(floor, node_id);

                local.order_table.set_cab_order(floor, node_id, cab_order);
                local.order_table.set_seen_by_cab(floor, node_id, local.self_id);
                local.counters.set_cab_order(floor, node_id, incoming_ct);
            } 
            else if incoming_ct == local_ct {
                local.order_table.set_seen_by_cab(floor, node_id, incoming.self_id);
            }
        }
    }
}

fn merge_peer_availability(local: &mut WorldView, incoming: &WorldView) {
    for node_id in 0..N_NODES {
        let local_ct    = local.counters.get_peer_availability(node_id);
        let incoming_ct = incoming.counters.get_peer_availability(node_id);

        if incoming_ct > local_ct {
            let is_available = incoming.peer_monitor.availability[node_id];

            local.peer_monitor.set(node_id, is_available);
            local.counters.set_peer_availability(node_id, incoming_ct);
        }
    }
}

fn merge_elevators(local: &mut WorldView, incoming: &WorldView) {
    for node_id in 0..N_NODES {
        let local_ct    = local.counters.get_elevator(node_id);
        let incoming_ct = incoming.counters.get_elevator(node_id);

        if incoming_ct > local_ct {
            let elevator = incoming.node_states.get(node_id);
            let _ = local.node_states.set(node_id, elevator);

            local.counters.set_elevator(node_id, incoming_ct);
        }
    }
}
