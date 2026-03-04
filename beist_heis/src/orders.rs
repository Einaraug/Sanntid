use crate::elev_algo::elevator::{Button, N_FLOORS};
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
    pub id: usize,
    pub seen_by: [bool; N_NODES],
}

impl HallOrder {
    pub fn new() -> Self {
        Self {
            state: OrderState::None,
            id: UNASSIGNED_NODE,
            seen_by: [false; N_NODES],
        }
    }
    pub fn update_state(&mut self, state: OrderState){
        self.state = state
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
    pub fn update_state(&mut self, state: OrderState){
        self.state = state
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

    // --- Hall state ---
    // ---- GETTERS ----
    pub fn get_hall_orders(&self) -> &[[HallOrder; 2]; N_FLOORS] {
        &self.hall
    }
    pub fn get_hall_order(&self, floor: usize, btn_idx: usize) -> HallOrder{
        let hall_orders = self.get_hall_orders();
        hall_orders[floor][btn_idx]
    }

    pub fn get_cab_orders(&self) -> &[[CabOrder; N_NODES]; N_FLOORS] {
        &self.cab
    }

    pub fn get_cab_order(&self, floor: usize, node_id: usize) -> CabOrder{
        let cab_orders = self.get_cab_orders();
        cab_orders[floor][node_id]
    }

    pub fn get_hall_state(&self, floor: usize, button: Button) -> OrderState {
        match button {
            Button::HallUp   => self.hall[floor][0].state,
            Button::HallDown => self.hall[floor][1].state,
            Button::Cab      => panic!("Use get_cab_state for cab orders"),
        }
    }

    pub fn get_hall_seen_by(&self, floor: usize, button: Button, node_id: usize) -> bool {
        match button {
            Button::HallUp   => self.hall[floor][0].seen_by[node_id],
            Button::HallDown => self.hall[floor][1].seen_by[node_id],
            Button::Cab      => panic!("get_hall_seen_by called with Button::Cab"),
        }
    }

    pub fn clear_hall(&mut self, floor: usize, button: Button) {
        self.update_hall_state(floor, button, OrderState::None);
        self.update_hall_id(floor, button, UNASSIGNED_NODE);
        match button {
            Button::HallUp   => self.hall[floor][0].seen_by = [false; N_NODES],
            Button::HallDown => self.hall[floor][1].seen_by = [false; N_NODES],
            Button::Cab      => panic!("clear_hall called with Button::Cab"),
        }
    }

    // --- Cab state ---

    pub fn update_cab_state(&mut self, floor: usize, node_id: usize, state: OrderState) {
        self.cab[floor][node_id].state = state;
    }

    pub fn update_cab_seen_by(&mut self, floor: usize, node_id: usize, seen_by_node: usize, seen: bool) {
        self.cab[floor][node_id].seen_by[seen_by_node] = seen;
    }

    pub fn get_cab_state(&self, floor: usize, node_id: usize) -> OrderState {
        self.cab[floor][node_id].state
    }

    pub fn get_cab_seen_by(&self, floor: usize, node_id: usize, seen_by_node: usize) -> bool {
        self.cab[floor][node_id].seen_by[seen_by_node]
    }

    pub fn clear_cab(&mut self, floor: usize, node_id: usize) {
        self.cab[floor][node_id].state = OrderState::None;
        self.cab[floor][node_id].seen_by = [false; N_NODES];
    }

    // --- Button press ---
    pub fn on_button_press(&mut self, floor: usize, button: Button, node_id: usize) {
        match button {
            Button::HallUp | Button::HallDown => {
                self.update_hall_state(floor, button, OrderState::Unconfirmed);
                self.update_hall_id(floor, button, UNASSIGNED_NODE);
            }
            Button::Cab => {
                self.update_cab_state(floor, node_id, OrderState::Unconfirmed);
            }
        }
    }
}