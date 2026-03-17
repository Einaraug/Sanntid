use crate::elev_algo::elevator::{Button, N_FLOORS, N_BUTTONS};
use crate::world_view::{N_NODES, N_DIRS};
use crate::counters::Change;
use serde::{Serialize, Deserialize};

pub const UNASSIGNED: usize = N_NODES + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderState {
    None,
    Unconfirmed,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HallOrder {
    pub state: OrderState,
    pub assigned_to: usize,
    pub seen_by: [bool; N_NODES],
}

impl HallOrder {
    pub fn new() -> Self {
        Self {
            state: OrderState::None,
            assigned_to: UNASSIGNED,
            seen_by: [false; N_NODES],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}

// Holds all orders for all nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTable {
    pub hall: [[HallOrder; N_DIRS]; N_FLOORS],
    pub cab: [[CabOrder; N_NODES]; N_FLOORS],
}

impl OrderTable {
    pub fn new() -> Self {
        Self {
            hall: [[HallOrder::new(); N_DIRS]; N_FLOORS],
            cab: [[CabOrder::new(); N_NODES]; N_FLOORS],
        }
    }

    // Getters
    pub fn get_hall_order(&self, floor: usize, btn_id: usize) -> HallOrder {
        self.hall[floor][btn_id]
    }
    pub fn get_cab_order(&self, floor: usize, node_id: usize) -> CabOrder {
        self.cab[floor][node_id]
    }

    // Setters
    pub fn set_hall_order(&mut self, floor: usize, btn: Button, order: HallOrder) {
        self.hall[floor][btn.to_index()] = order;
    }
    pub fn set_cab_order(&mut self, floor: usize, node_id: usize, order: CabOrder) {
        self.cab[floor][node_id] = order;
    }
    pub fn set_hall_order_state(&mut self, floor: usize, btn: Button, state: OrderState) {
        self.hall[floor][btn.to_index()].state = state;
    }
    pub fn set_cab_order_state(&mut self, floor: usize, node_id: usize, state: OrderState) {
        self.cab[floor][node_id].state = state;
    }
    pub fn set_hall_order_assigned_to(&mut self, floor: usize, btn: Button, node_id: usize) {
        self.hall[floor][btn.to_index()].assigned_to = node_id;
    }
    pub fn set_seen_by_hall(&mut self, floor: usize, btn: Button, observer_node_id: usize) {
        self.hall[floor][btn.to_index()].seen_by[observer_node_id] = true;
    }
    pub fn set_seen_by_cab(&mut self, floor: usize, node_id: usize, observer_node_id: usize) {
        self.cab[floor][node_id].seen_by[observer_node_id] = true;
    }

    // Order lifecycle

    // Sets OrderState to Unconfirmed from None
    pub fn on_btn_press(&mut self, floor: usize, btn: Button, self_id: usize) -> Vec<Change> {
        match btn {
            Button::HallUp | Button::HallDown => {
                if self.hall[floor][btn.to_index()].state == OrderState::Confirmed {
                    return vec![];
                }
                self.set_hall_order_state(floor, btn, OrderState::Unconfirmed);
                self.set_hall_order_assigned_to(floor, btn, UNASSIGNED);
                self.set_seen_by_hall(floor, btn, self_id);
                vec![Change::HallOrder{floor, btn}]
            }
            Button::Cab => {
                if self.cab[floor][self_id].state == OrderState::Confirmed {
                    return vec![];
                }
                self.set_cab_order_state(floor, self_id, OrderState::Unconfirmed);
                self.set_seen_by_cab(floor, self_id, self_id);
                vec![Change::CabOrder{floor, node_id: self_id}]
            }
        }
    }

    // If an order is seen by all available nodes, sets OrderState to Confirmed from Unconfirmed
    pub fn try_confirm_orders(&mut self, peer_availability: &[bool; N_NODES]) -> Vec<Change> {
        let mut changes = Vec::new();
        
        for floor in 0..N_FLOORS {
            for node_id in 0..N_NODES {
                let cab_order = self.cab[floor][node_id];

                if cab_order.state == OrderState::Unconfirmed && is_seen_by_all(&cab_order.seen_by, peer_availability) {
                    self.set_cab_order_state(floor, node_id, OrderState::Confirmed);
                    changes.push(Change::CabOrder{floor, node_id});
                }
            }
            for btn in [Button::HallUp, Button::HallDown] {
                let hall_order = self.hall[floor][btn.to_index()];

                if hall_order.state == OrderState::Unconfirmed && is_seen_by_all(&hall_order.seen_by, peer_availability) {
                    self.set_hall_order_state(floor, btn, OrderState::Confirmed);
                    changes.push(Change::HallOrder{floor, btn});
                }
            }
        }
        changes
    }


    pub fn clear_hall_order(&mut self, floor: usize, btn: Button) -> Vec<Change> {
        self.hall[floor][btn.to_index()] = HallOrder::new();
        vec![Change::HallOrder{floor, btn}]
    }
    pub fn clear_cab_order(&mut self, floor: usize, node_id: usize) -> Vec<Change> {
        self.cab[floor][node_id] = CabOrder::new();
        vec![Change::CabOrder {floor, node_id}]
    }

    pub fn assign_order_to(&mut self, floor: usize, btn: Button, node_id: usize) -> Vec<Change> {
        self.set_hall_order_assigned_to(floor, btn, node_id);
        vec![Change::HallOrder{floor, btn}]
    }

    // If a node becomes unable to handle orders, its orders must be unassigned
    pub fn unassign_orders_for(&mut self, node_id: usize) -> Vec<Change> {
        let mut changes = Vec::new();

        for floor in 0..N_FLOORS {
            for btn in [Button::HallUp, Button::HallDown] {
                let hall_order = self.hall[floor][btn.to_index()];

                if hall_order.assigned_to == node_id && hall_order.state == OrderState::Confirmed {
                    self.set_hall_order_assigned_to(floor, btn, UNASSIGNED);
                    changes.push(Change::HallOrder{floor, btn});
                }
            }
        }
        changes
    }

    // Converts OrderTable to the format required by FSM
    pub fn convert_to_requests(&self, self_id: usize) -> [[bool; N_BUTTONS]; N_FLOORS] {
        let mut requests = [[false; N_BUTTONS]; N_FLOORS];

        for floor in 0..N_FLOORS {
            if self.cab[floor][self_id].state == OrderState::Confirmed {
                requests[floor][Button::Cab.to_index()] = true;
            }

            for btn in [Button::HallUp, Button::HallDown] {
                let hall_order = self.hall[floor][btn.to_index()];
                if hall_order.state == OrderState::Confirmed && hall_order.assigned_to == self_id {
                    requests[floor][btn.to_index()] = true;
                }
            }
        }
        requests
    }
}

// Helper function for try_confirm_orders
fn is_seen_by_all(seen_by: &[bool; N_NODES], peer_availability: &[bool; N_NODES]) -> bool {
    for node_id in 0..N_NODES {
        if peer_availability[node_id] && !seen_by[node_id] {
            return false;
        }
    }
    true
}
