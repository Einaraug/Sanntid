use serde::{Serialize, Deserialize};
use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::world_view::{N_NODES, N_DIRS};

// Describes which part of the WorldView was mutated by an operation
pub enum Change {
    Elevator {node_id: usize},
    PeerAvailability {node_id: usize},
    HallOrder {floor: usize, btn: Button},
    CabOrder {floor: usize, node_id: usize},
}

// Version counters implementing a last-write-wins CRDT (Conflict-free Replicated Data Type).
// Each field is a monotonically increasing version number that tracks changes in the WorldView.
// When merging WorldViews, the data with the highest corresponding counter overwrites the older data.
// The system must restart before the 64-bit counter overflows.
#[derive(Clone, Serialize, Deserialize)]
pub struct Counters {
    elevator: [u64; N_NODES],
    peer_availability: [u64; N_NODES],
    hall_order: [[u64; N_DIRS]; N_FLOORS],
    cab_order: [[u64; N_NODES]; N_FLOORS],
}

impl Counters {
    pub fn new(self_id: usize) -> Self {
        let mut peer_availability = [0; N_NODES];
        peer_availability[self_id] = 1;
        Self {
            elevator: [0; N_NODES],
            peer_availability,
            hall_order: [[0; N_DIRS]; N_FLOORS],
            cab_order: [[0; N_NODES]; N_FLOORS],
        }
    }

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
                Change::HallOrder {floor, btn} => self.inc_hall_order(floor, btn),
                Change::CabOrder {floor, node_id} => self.inc_cab_order(floor, node_id),
                Change::Elevator {node_id} => self.inc_elevator(node_id),
                Change::PeerAvailability {node_id} => self.inc_peer_availability(node_id),
            }
        }
    }
}
