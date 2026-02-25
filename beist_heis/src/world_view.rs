use crate::elev_algo::elevator::{Button, Elevator, N_FLOORS};
use crate::orders::*;

pub const N_NODES: usize = 3;
type ElevId = u32;


#[derive(Clone)]
pub struct ElevatorMap {
    elevator: [Elevator; N_NODES],
}
impl ElevatorMap {
    pub fn new() -> Self {
        Self {
            elevator: [Elevator::new(); N_NODES], //WorldView owns the elevators, so we can initialize them here. A node only owns its own id.
        }
    }
    pub fn get_elevator(&self, node_id: usize) -> &Elevator {
        &self.elevator[node_id]
    }
    pub fn update_elevator(&mut self, node_id: usize, elevator: Elevator) {
        self.elevator[node_id] = elevator;
    }
}

#[derive(Clone)]  
pub struct PeerAvailability{
    peer_availability: [bool; N_NODES],
}
impl PeerAvailability {
    pub fn new() -> Self {
        Self {
            peer_availability: [false; N_NODES], //Initially, all peers are unavailable until they announce themselves. WorldView owns the availability, so we can initialize it here.
        }
    }
    pub fn set_availability(&mut self, node_id: usize, available: bool) {
        self.peer_availability[node_id] = available;
    }
    pub fn is_available(&self, node_id: usize) -> bool {
        self.peer_availability[node_id]
    }
}

pub struct WorldView {
    self_id: i32,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counts: Counters,
}
pub impl WorldView {
    pub fn new(self_id: i32) -> Self {
        Self {
            self_id: self_id,
            elevator_map: ElevatorMap::new(),
            peer_availability: PeerAvailability::new(),
            order_table: OrderTable::new(),
            counts: Counters::new(),
        }
    }

    // Getters :: Should the return type be references?
    pub fn get_self_id(&self) -> i32 {
        self.self_id
    }

    pub fn get_elevator_map(&self) -> &ElevatorMap {
        &self.elevator_map
    }

    pub fn get_peer_availability(&self) -> &PeerAvailability {
        &self.peer_availability
    }

    pub fn get_order_table(&self) -> &OrderTable {
        &self.order_table
    }

    pub fn get_counts(&self) -> &Counters {
        &self.counts
    }

    // Setters
    pub fn update_elevator(&mut self, node_id: usize, elevator: Elevator) {
        self.elevator_map.update_elevator(node_id, elevator);
    }

    pub fn set_peer_availability(&mut self, node_id: usize, available: bool) {
        self.peer_availability.set_availability(node_id, available);
    }

    pub fn update_order_table(&mut self, floor: usize, button: Button, node_id: usize, state: OrderState) {
       self.order_table.update_hall_state(floor, button, state);
       self.order_table.update_hall_id(floor, button, node_id);
    }

    pub fn update_cab_order(&mut self, floor: usize, node_id: usize, state: OrderState) {
        self.order_table.update_cab(floor, node_id, state);
    }
}