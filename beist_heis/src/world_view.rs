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

pub struct WorldView {
    self_id: i32,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counts: Counters,
}

