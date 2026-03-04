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


//TODO: Generalize names
pub struct WorldView {
    self_id: i32,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counters: Counters,
}
impl WorldView {
    pub fn new(self_id: i32) -> Self {
        Self {
            self_id: self_id,
            elevator_map: ElevatorMap::new(),
            peer_availability: PeerAvailability::new(),
            order_table: OrderTable::new(),
            counters: Counters::new(),
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

    pub fn get_counters(&self) -> &Counters {
        &self.counters
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

    pub fn merge(&mut self, &incoming: WorldView){
        merge_hall_orders(self, incoming);
        merge_cab_orders(self, incoming);
        merge_peer_status(self, incoming);
        merge_elevator(self, incoming);
    }
}


//Helper functions for merging worldviews
fn merge_hall_orders(local: &mut WorldView, incoming: &WorldView){
    for floor in 0..N_FLOORS{
        for button in [Button::HallUp, Button::HallDown]{
            let local_ct = local.counters.get_hall_order(floor, button);
            let incoming_ct = incoming.counters.get_hall_order(floor, button);
            if incoming_ct > local_ct {
                let incoming_state = incoming.order_table.get_hall_state(floor, button);
                local.order_table.update_hall(floor, button, incoming_state);
                local.counters.set_hall_order(floor, button, incoming_ct);
            }
            else if incoming_ct == local_ct {
                let incoming_id = incoming.self_id as usize;
                local.order_table.update_hall_seen_by(floor, button, incoming_id, true);
            }
        }
    }
}

fn merge_cab_orders(local: &mut WorldView, incoming: &WorldView) {
    for floor in 0..N_FLOORS {
        for node in 0..N_NODES {
            let local_ct = local.counters.get_cab_order(floor, node);
            let incoming_ct = incoming.counters.get_cab_order(floor, node);
            if incoming_ct > local_ct {
                let incoming_state = incoming.order_table.get_cab_state(floor, node);
                let incoming_id = incoming.self_id as usize;
                local.order_table.update_cab(floor, node, incoming_state);
                local.counters.set_cab_order(floor, incoming_id, incoming_ct);
            }
            else if incoming_ct == local_ct {
                let incoming_id = incoming.self_id as usize;
                local.order_table.update_cab_seen_by(floor, incoming_id, true);
            }
        }
    }
}

fn merge_peer_status(local: &mut WorldView, incoming: &WorldView) {
    for node in 0..N_NODES {
        let local_ct = local.counters.get_peer_status(node);
        let incoming_ct = incoming.counters.get_peer_status(node);
        if incoming_ct > local_ct {
            let incoming_availability = incoming.get_peer_availability().is_available(node);
            let incoming_id = incoming.self_id as usize;
            local.set_peer_availability(node, incoming_availability);
            local.counters.set_peer_status(incoming_id, incoming_ct);
        }
    }
}

fn merge_elevator(local: &mut WorldView, incoming: &WorldView) {
    for node in 0..N_NODES {
        let local_ct = local.counters.get_elevator(node);
        let incoming_ct = incoming.counters.get_elevator(node);
        if incoming_ct > local_ct {
            let incoming_elevator = *incoming.get_elevator_map().get_elevator(node);
            let incoming_id = incoming.self_id as usize;
            local.update_elevator(node, incoming_elevator);
            local.counters.set_elevator(incoming_id, incoming_ct);
        }   
    }
}