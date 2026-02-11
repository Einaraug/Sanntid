use std::collections::HashMap;

type ElevId = u32;
const N_FLOORS: usize = 4;
const N_HALLORDERS: usize = (N_FLOORS - 1) * 2;

pub struct ElevatorState {
    current_floor: i32,
    direction: String,
    // Add other fields you need
}

pub struct ElevatorMap {
    elevator: HashMap<ElevId, ElevatorState>,
}

pub struct PeerAvailability{
    peer_availability: HashMap<ElevId, bool>
}
#[derive(Clone)]
pub struct HallOrder{
    floor: i32,
    dir: ElevatorDir, //TODO: Define
    count: u64
}
pub struct WorldView {
    self_id: i32,
    self_state: ElevatorState, //TODO: IMPLEMENT
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    hall_table: Vec<HallOrder>,
}
