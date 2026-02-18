use std::collections::HashMap;
use crate::elev_algo::elevator::{Dirn, Behaviour, N_FLOORS, N_BUTTONS};

type ElevId = u32;
pub struct ElevatorState {
    current_floor: i32,
    direction: String,
    // Add other fields you need
}

pub struct ElevatorMap {
    elevator: HashMap<ElevId, ElevatorState>,
}

pub struct PeerAvailability{
    peer_availability: HashMap<ElevId, bool>,

}
#[derive(Clone)]
pub struct HallOrder{
    floor: i32,
    dir: Dirn, //TODO: Define
    state: enum
}

#[derive(Clone)]
pub struct Counters{
    ct_hall_order: Vec<u64>, //Size N_HALLORDERS
    ct_cab_order: Vec<u64>, //Size: N_caborders * N_nodes
    ct_peer_status: Vec<u64>, //Size: N_nodes
    ct_floor: Vec<u64>, //SIZE N_nodes, change at idx i implies change at floor for node i
    ct_dir: Vec<u64>,
    ct_state: Vec<u64>,
    ct_obstruction: Vec<u64>
}

pub struct WorldView {
    self_id: i32,
    self_state: ElevatorState, //TODO: IMPLEMENT
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    hall_table: Vec<HallOrder>,
    counts: Counters,
}


