#![allow(dead_code, non_snake_case)]
use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::world_view::N_NODES;

#[derive(Clone)]
pub struct Counters{
    hall_order: [[u64; 2]; N_FLOORS],
    cab_order: [[u64; N_NODES]; N_FLOORS],
    peer_status: [u64; N_NODES],
    elevator: [u64; N_NODES],

}
impl Counters{
    pub fn new() -> Self {
        Self {
            hall_order: [[0; 2]; N_FLOORS],
            cab_order: [[0; N_NODES]; N_FLOORS],
            peer_status: [0; N_NODES],
            elevator: [0; N_NODES],

        }
    }

    // ── Increment ────────────────────────────────────────────────────────────
    pub fn inc_hall_order(&mut self, floor: usize, button: Button) {
        match button {
            Button::HallUp => self.hall_order[floor][0] += 1,
            Button::HallDown => self.hall_order[floor][1] += 1,
            Button::Cab => panic!("inc_hall_order called with Button::Cab"),
        }
    }
    pub fn inc_cab_order(&mut self, floor: usize, node_id: usize) { self.cab_order[floor][node_id] += 1; }
    pub fn inc_peer_status(&mut self, node_id: usize) { self.peer_status[node_id] += 1; }
    pub fn inc_elevator(&mut self, node_id: usize) { self.elevator[node_id] += 1; }
   
    // ── Getters ──────────────────────────────────────────────────────────────
    pub fn get_hall_order(&self, floor: usize, button: Button) -> u64 {
        match button {
            Button::HallUp => self.hall_order[floor][0],
            Button::HallDown => self.hall_order[floor][1],
            Button::Cab => panic!("get_hall_order called with Button::Cab"),
        }
    }
    pub fn get_cab_order(&self, floor: usize, node_id: usize) -> u64 { self.cab_order[floor][node_id]}
    pub fn get_peer_status(&self, node_id: usize) -> u64 { self.peer_status[node_id]}
    pub fn get_elevator(&self, node_id: usize) -> u64 { self.elevator[node_id]}
   
    // ── Setters ──────────────────────────────────────────────────────────────
    pub fn set_hall_order(&mut self, floor: usize, button: Button, value: u64) {
        match button {
            Button::HallUp => self.hall_order[floor][0] = value,
            Button::HallDown => self.hall_order[floor][1] = value,
            Button::Cab => panic!("set_hall_order called with Button::Cab"),
        }
    }
    pub fn set_cab_order(&mut self, floor: usize, node_id: usize, value: u64) { self.cab_order[floor][node_id] = value; }
    pub fn set_peer_status(&mut self, node_id: usize, value: u64) { self.peer_status[node_id] = value; }
    pub fn set_elevator(&mut self, node_id: usize, value: u64) { self.elevator[node_id] = value; }
    
}






