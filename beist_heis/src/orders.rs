use crate::elev_algo::elevator::{N_FLOORS};
use crate::world_view::N_NODES;
pub const UNASSIGNED_NODE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderState {
    None,
    Unconfirmed,
    Confirmed,
}

#[derive(Debug, Clone, Copy)]
pub struct HallOrder {
    pub state: OrderState,
    pub node_id: usize,
    pub seen_by: [bool; N_NODES],
}

impl HallOrder {
    pub fn new() -> Self {
        Self {
            state: OrderState::None,
            node_id: UNASSIGNED_NODE,
            seen_by: [false; N_NODES],
        }
    }
    pub fn get_state(&self) -> OrderState{
        self.state
    }
    pub fn set_state(&mut self, state: OrderState){
        self.state = state
    }

    pub fn get_node_id(&self) -> usize {
        self.node_id
    }
    
    pub fn set_node_id(&mut self, node_id: usize) {
        self.node_id = node_id;
    }

    pub fn get_seen_by(&self) -> [bool; N_NODES]{
        self.seen_by
    }
    pub fn set_seen_by(&mut self, node_id: usize){
        self.seen_by[node_id] = true;
    }

    pub fn clear(&mut self){
         *self = HallOrder::new();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CabOrder {
    pub state: OrderState,
    pub seen_by: [bool; N_NODES],
}
impl CabOrder {
    pub fn new() -> Self {
        Self {
            state: OrderState::None,
            seen_by: [false; N_NODES],
        }
    }

    pub fn get_state(&self) -> OrderState {
        self.state
    }
    pub fn set_state(&mut self, state: OrderState) {
        self.state = state;
    }

    pub fn get_seen_by(&self) -> [bool; N_NODES] {
        self.seen_by
    }
    pub fn set_seen_by(&mut self, node_id: usize) {
        self.seen_by[node_id] = true;
    }

    pub fn clear(&mut self){
        *self = CabOrder::new();
    }
}

#[derive(Debug, Clone)]
pub struct OrderTable {
    pub hall: [[HallOrder; 2]; N_FLOORS],
    pub cab:  [[CabOrder; N_NODES]; N_FLOORS],
}

impl OrderTable {
    pub fn new() -> Self {
        Self {
            hall: [[HallOrder::new(); 2]; N_FLOORS],
            cab:  [[CabOrder::new(); N_NODES]; N_FLOORS],
        }
    }

    // ---- GETTERS ----
    pub fn get_hall_order(&self, floor: usize, btn_id: usize) -> HallOrder {
        self.hall[floor][btn_idx]  // copy, for reading
    }
    pub fn get_hall_order_mut(&mut self, floor: usize, btn_id: usize) -> &mut HallOrder {
        &mut self.hall[floor][btn_idx]  // mutable ref, for mutating
    }

    pub fn get_cab_order(&self, floor: usize, node_id: usize) -> CabOrder {
        self.cab[floor][node_id]
    }
    pub fn get_cab_order_mut(&mut self, floor: usize, node_id: usize) -> &mut CabOrder {
        &mut self.cab[floor][node_id]
    }
}
