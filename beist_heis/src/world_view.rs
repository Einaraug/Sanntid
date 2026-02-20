use std::collections::HashMap;
use crate::elev_algo::elevator::{Button, Elevator, N_FLOORS};
use crate::orders::*;

pub const N_NODES: usize = 3;
type ElevId = u32;

pub struct ElevatorMap {
    elevator: HashMap<ElevId, Elevator>,
}

pub struct PeerAvailability{
    peer_availability: HashMap<ElevId, bool>,
}


//Move this to its own module?
#[derive(Clone)]
pub struct Counters{
    ct_hall_order:  [[u64; 2]; N_FLOORS],
    ct_cab_order:   [[u64; N_NODES]; N_FLOORS],
    ct_peer_status: [u64; N_NODES],
    ct_floor:       [u64; N_NODES],
    ct_dir:         [u64; N_NODES],
    ct_state:       [u64; N_NODES],
    ct_obstruction: [u64; N_NODES],
}
impl Counters{
    pub fn new() -> Self {
        Self {
            ct_hall_order:  [[0; 2]; N_FLOORS],
            ct_cab_order:   [[0; N_NODES]; N_FLOORS],
            ct_peer_status: [0; N_NODES],
            ct_floor:       [0; N_NODES],
            ct_dir:         [0; N_NODES],
            ct_state:       [0; N_NODES],
            ct_obstruction: [0; N_NODES],
        }
    }
    // ── Increment ────────────────────────────────────────────────────────────
    pub fn inc_hall_order(&mut self, floor: usize, button: Button) {
        match button {
            Button::HallUp   => self.ct_hall_order[floor][0] += 1,
            Button::HallDown => self.ct_hall_order[floor][1] += 1,
            Button::Cab      => panic!("inc_hall_order called with Button::Cab"),
        }
    }
    pub fn inc_cab_order  (&mut self, floor: usize, node_id: usize) { self.ct_cab_order[floor][node_id] += 1; }
    pub fn inc_peer_status(&mut self, node_id: usize) { self.ct_peer_status[node_id] += 1; }
    pub fn inc_floor      (&mut self, node_id: usize) { self.ct_floor[node_id]        += 1; }
    pub fn inc_dir        (&mut self, node_id: usize) { self.ct_dir[node_id]          += 1; }
    pub fn inc_state      (&mut self, node_id: usize) { self.ct_state[node_id]        += 1; }
    pub fn inc_obstruction(&mut self, node_id: usize) { self.ct_obstruction[node_id]  += 1; }

    // ── Getters ──────────────────────────────────────────────────────────────
    pub fn get_hall_order(&self, floor: usize, button: Button) -> u64 {
        match button {
            Button::HallUp   => self.ct_hall_order[floor][0],
            Button::HallDown => self.ct_hall_order[floor][1],
            Button::Cab      => panic!("get_hall_order called with Button::Cab"),
        }
    }
    pub fn get_cab_order  (&self, floor: usize, node_id: usize) -> u64 { self.ct_cab_order[floor][node_id] }
    pub fn get_peer_status(&self, node_id: usize) -> u64 { self.ct_peer_status[node_id] }
    pub fn get_floor      (&self, node_id: usize) -> u64 { self.ct_floor[node_id]       }
    pub fn get_dir        (&self, node_id: usize) -> u64 { self.ct_dir[node_id]         }
    pub fn get_state      (&self, node_id: usize) -> u64 { self.ct_state[node_id]       }
    pub fn get_obstruction(&self, node_id: usize) -> u64 { self.ct_obstruction[node_id] }
}

pub struct WorldView {
    self_id: i32,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counts: Counters,
}

