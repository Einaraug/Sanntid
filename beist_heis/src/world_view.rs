use crate::elev_algo::elevator::{Button, Elevator, N_BUTTONS, N_FLOORS};
use crate::orders::*;
use crate::counters::*;


pub const N_NODES: usize = 3;
type ElevId = usize;

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
    pub fn get(&self, node_id: usize) -> &Elevator {
        &self.elevator[node_id]
    }
    pub fn set(&mut self, node_id: usize, elevator: Elevator) {
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
    pub fn get(&self, node_id: usize) -> bool {
        self.peer_availability[node_id]
    }
    pub fn set(&mut self, node_id: usize, available: bool) {
        self.peer_availability[node_id] = available;
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, bool)> {
        self.peer_availability.iter().copied().enumerate()
    }
}
//TODO: Generalize names
pub struct WorldView {
    self_id: usize,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counters: Counters,
}
impl WorldView {
    pub fn new(self_id: usize) -> Self {
        Self {
            self_id: self_id,
            elevator_map: ElevatorMap::new(),
            peer_availability: PeerAvailability::new(),
            order_table: OrderTable::new(),
            counters: Counters::new(),
        }
    }

    // Getters :: Should the return type be references?
    pub fn get_self_id(&self) -> &usize {
        &self.self_id
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

    // --BUTTON PRESS HANDLER --
    pub fn on_button_press(&mut self, floor: usize, button: Button) {
        match button {
            Button::HallUp | Button::HallDown => {
                let order = self.order_table.get_hall_order_mut(floor, button as usize);
                order.set_state(OrderState::Unconfirmed);
                order.set_node_id(UNASSIGNED_NODE);
            }
            Button::Cab => {
                let order = self.order_table.get_cab_order_mut(floor, self.self_id);
                order.set_state(OrderState::Unconfirmed);
            }
        }
    }

    pub fn is_all_acked(&self, seen_by: &[bool; N_NODES]) -> bool {
        for (node_id, available) in self.peer_availability.iter() {
            if available && !seen_by[node_id] {
                return false;
            }
        }
        true
    }
    pub fn modify_order_states(&mut self) {
        for floor in 0..N_FLOORS {
        // HANDLE CAB ORDERS
        for node_id in 0..N_NODES {
            let seen_by = self.order_table.get_cab_order(floor, node_id).get_seen_by();
            if self.is_all_acked(&seen_by) {
                self.order_table.get_cab_order_mut(floor, node_id).set_state(OrderState::Confirmed);
            }
        }

        // HANDLE HALL ORDERS
        for button in [Button::HallUp, Button::HallDown] {
            let seen_by = self.order_table.get_hall_order(floor, button as usize).get_seen_by();
            if self.is_all_acked(&seen_by) {
                self.order_table.get_hall_order_mut(floor, button as usize).set_state(OrderState::Confirmed);
                }
            }
        }
    }
    

    pub fn run_world_view(&mut self){
        //Init func should be called outside.

        //while(1){}
            //Recieve incoming worldview
            //Update own world_view using merge_worldviews func()

            //Check for "order to be handled" -->
            //Poll buttons  + increment state counters
            //If new order - handle using orders.rs
            //If stop / obstruction. Update state and pass to fsm to handle states
            //Increment counters
            
            //pass itself as message to network thread?
            //Send action to fsm
            //Send action to lights
    }
    pub fn merge(&mut self, incoming: &WorldView){
        //Should be atomic?
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
            let local_order = local.order_table.get_hall_order_mut(floor, button as usize);

            //Incoming world_view with more recent state
            if incoming_ct > local_ct {
                let incoming_order = incoming.order_table.get_hall_order(floor, button as usize);
                *local_order = incoming_order;
                local.counters.set_hall_order(floor, button, incoming_ct);
            }

            else if incoming_ct == local_ct {
                let incoming_id = incoming.self_id;
                local_order.set_seen_by(incoming_id);
            }
        }
    }
}

fn merge_cab_orders(local: &mut WorldView, incoming: &WorldView) {
    for floor in 0..N_FLOORS {
        for node_id in 0..N_NODES {
            let local_ct = local.counters.get_cab_order(floor, node_id);
            let incoming_ct = incoming.counters.get_cab_order(floor, node_id);
            let local_order = local.order_table.get_cab_order_mut(floor, node_id);

            if incoming_ct > local_ct {
                let incoming_order = incoming.order_table.get_cab_order(floor, node_id);
                *local_order = incoming_order;
                local.counters.set_cab_order(floor, node_id, incoming_ct);
            }
            else if incoming_ct == local_ct {
                let incoming_id = incoming.self_id;
                local_order.set_seen_by(incoming_id);
            }
        }
    }
}

fn merge_peer_status(local: &mut WorldView, incoming: &WorldView) {
    let incoming_availability = incoming.get_peer_availability();
    for node in 0..N_NODES {
        let local_ct = local.counters.get_peer_status(node);
        let incoming_ct = incoming.counters.get_peer_status(node);

        if incoming_ct > local_ct {
            let is_available: bool = incoming_availability.get(node);

            local.peer_availability.set(node, is_available);
            local.counters.set_peer_status(node, incoming_ct);
        }
    }
}

fn merge_elevator(local: &mut WorldView, incoming: &WorldView) {
    for node in 0..N_NODES {
        let local_ct = local.counters.get_elevator(node);
        let incoming_ct = incoming.counters.get_elevator(node);
        if incoming_ct > local_ct {
            let incoming_elevator = *incoming.elevator_map.get(node);
            local.elevator_map.set(node, incoming_elevator);
            local.counters.set_elevator(node, incoming_ct);
        }
    }
}