use crate::elev_algo::elevator::{Button, Elevator, N_BUTTONS, N_FLOORS};
use crate::orders::*;

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
    pub fn get_availability(&self, node_id: usize) -> bool {
        self.peer_availability[node_id]
    }
    pub fn update_availability(&mut self, node_id: usize, available: bool) {
        self.peer_availability[node_id] = available;
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, bool)> {
        self.peer_availability.iter().copied().enumerate()
    }
}


//TODO: Generalize names
pub struct WorldView {
    self_id: u32,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counters: Counters,
}
impl WorldView {
    pub fn new(self_id: u32) -> Self {
        Self {
            self_id: self_id,
            elevator_map: ElevatorMap::new(),
            peer_availability: PeerAvailability::new(),
            order_table: OrderTable::new(),
            counters: Counters::new(),
        }
    }

    // Getters :: Should the return type be references?
    pub fn get_self_id(&self) -> u32 {
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
        &self.counters
    }

    pub fn is_all_acked(&self, seen_by: &[bool; N_NODES]) -> bool {
        for (node_id, available) in self.get_peer_availability().iter() {
            if available && !seen_by[node_id] {
                return false;
            }
        }
        true
    }

    pub fn modify_order_states(&mut self){
        let order_table = self.get_order_table();        
        for floor in 0..N_FLOORS{

            //HANDLE CAB ORDERS
            for node_id in 0..N_NODES{
                let mut cab_order = order_table.get_cab_order(floor, node_id);
                let seen_by = cab_order.seen_by;
                if self.is_all_acked(&seen_by){
                    cab_order.update_state(OrderState::Confirmed);
                }
            }

            //HANDLE HALL ORDERS
            for btn_id in 0..2{
                let mut hall_order = order_table.get_hall_order(floor, btn_id);
                let seen_by = hall_order.seen_by;
                if self.is_all_acked(&seen_by){
                    hall_order.update_state(OrderState::Confirmed);
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
}






