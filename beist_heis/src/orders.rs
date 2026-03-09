use crate::elev_algo::elevator::{Button, N_FLOORS, N_BUTTONS};
use crate::world_view::N_NODES;
use crate::counters::Change;
use serde::{Serialize, Deserialize};

pub const UNASSIGNED_NODE: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderState {
    None,
    Unconfirmed,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HallOrder {
    pub state:   OrderState,
    pub node_id: usize,
    pub seen_by: [bool; N_NODES],
}

impl HallOrder {
    pub fn new() -> Self {
        Self {
            state:   OrderState::None,
            node_id: UNASSIGNED_NODE,
            seen_by: [false; N_NODES],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CabOrder {
    pub state:   OrderState,
    pub seen_by: [bool; N_NODES],
}

impl CabOrder {
    pub fn new() -> Self {
        Self {
            state:   OrderState::None,
            seen_by: [false; N_NODES],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    // ── Getters ───────────────────────────────────────────────────────────────
    pub fn get_hall_order(&self, floor: usize, btn_id: usize) -> HallOrder {
        self.hall[floor][btn_id]
    }
    pub fn get_cab_order(&self, floor: usize, node_id: usize) -> CabOrder {
        self.cab[floor][node_id]
    }

    // ── Pure setters (used by merge — no counter increments) ──────────────────
    pub fn set_hall_order_state(&mut self, floor: usize, btn: Button, state: OrderState) {
        self.hall[floor][btn.to_index()].state = state;
    }
    pub fn set_hall_order_node_id(&mut self, floor: usize, btn: Button, node_id: usize) {
        self.hall[floor][btn.to_index()].node_id = node_id;
    }
    pub fn set_cab_order_state(&mut self, floor: usize, node_id: usize, state: OrderState) {
        self.cab[floor][node_id].state = state;
    }
    pub fn set_seen_by_hall(&mut self, floor: usize, btn: Button, seen_by_id: usize) {
        self.hall[floor][btn.to_index()].seen_by[seen_by_id] = true;
    }
    pub fn set_seen_by_cab(&mut self, floor: usize, node_id: usize, seen_by_id: usize) {
        self.cab[floor][node_id].seen_by[seen_by_id] = true;
    }
    pub fn replace_hall_order(&mut self, floor: usize, btn: Button, order: HallOrder) {
        self.hall[floor][btn.to_index()] = order;
    }
    pub fn replace_cab_order(&mut self, floor: usize, node_id: usize, order: CabOrder) {
        self.cab[floor][node_id] = order;
    }

    // ── Order lifecycle (include counter increments) ───────────────────────────

    pub fn on_button_press(&mut self, floor: usize, button: Button, self_id: usize) -> Vec<Change> {
        match button {
            Button::HallUp | Button::HallDown => {
                self.set_hall_order_state(floor, button, OrderState::Unconfirmed);
                self.set_hall_order_node_id(floor, button, UNASSIGNED_NODE);
                self.set_seen_by_hall(floor, button, self_id);
                vec![Change::HallOrder { floor, button }]
            }
            Button::Cab => {
                self.set_cab_order_state(floor, self_id, OrderState::Unconfirmed);
                self.set_seen_by_cab(floor, self_id, self_id);
                vec![Change::CabOrder { floor, node_id: self_id }]
            }
        }
    }

    pub fn clear_hall_order(&mut self, floor: usize, button: Button) -> Vec<Change> {
        self.hall[floor][button.to_index()] = HallOrder::new();
        vec![Change::HallOrder { floor, button }]
    }

    pub fn clear_cab_order(&mut self, floor: usize, node_id: usize) -> Vec<Change> {
        self.cab[floor][node_id] = CabOrder::new();
        vec![Change::CabOrder { floor, node_id }]
    }

    pub fn assign_node_id(&mut self, floor: usize, btn: Button, node_id: usize) -> Vec<Change> {
        self.set_hall_order_node_id(floor, btn, node_id);
        vec![Change::HallOrder { floor, button: btn }]
    }

    pub fn unassign_orders_for(&mut self, node_id: usize) -> Vec<Change> {
        let mut changes = Vec::new();
        for floor in 0..N_FLOORS {
            for btn in [Button::HallUp, Button::HallDown] {
                let order = self.hall[floor][btn.to_index()];
                if order.node_id == node_id && order.state == OrderState::Confirmed {
                    self.set_hall_order_node_id(floor, btn, UNASSIGNED_NODE);
                    changes.push(Change::HallOrder { floor, button: btn });
                }
            }
        }
        changes
    }

    pub fn try_confirm_orders(&mut self, peer_availability: &[bool; N_NODES]) -> Vec<Change> {
        let mut changes = Vec::new();
        for floor in 0..N_FLOORS {
            for node_id in 0..N_NODES {
                let cab = self.cab[floor][node_id];
                if cab.state == OrderState::Unconfirmed
                    && is_all_acked(&cab.seen_by, peer_availability)
                {
                    self.set_cab_order_state(floor, node_id, OrderState::Confirmed);
                    changes.push(Change::CabOrder { floor, node_id });
                }
            }
            for btn in [Button::HallUp, Button::HallDown] {
                let hall = self.hall[floor][btn.to_index()];
                if hall.state == OrderState::Unconfirmed
                    && is_all_acked(&hall.seen_by, peer_availability)
                {
                    self.set_hall_order_state(floor, btn, OrderState::Confirmed);
                    changes.push(Change::HallOrder { floor, button: btn });
                }
            }
        }
        changes
    }

    // ── Pure computation ──────────────────────────────────────────────────────

    pub fn convert_to_requests(&self, self_id: usize) -> [[bool; N_BUTTONS]; N_FLOORS] {
        let mut requests = [[false; N_BUTTONS]; N_FLOORS];
        for floor in 0..N_FLOORS {
            requests[floor][Button::Cab.to_index()] =
                self.cab[floor][self_id].state == OrderState::Confirmed;
            for btn in [Button::HallUp, Button::HallDown] {
                let order = self.hall[floor][btn.to_index()];
                if order.state == OrderState::Confirmed && order.node_id == self_id {
                    requests[floor][btn.to_index()] = true;
                }
            }
        }
        requests
    }
}

fn is_all_acked(seen_by: &[bool; N_NODES], peer_availability: &[bool; N_NODES]) -> bool {
    for node_id in 0..N_NODES {
        if peer_availability[node_id] && !seen_by[node_id] {
            return false;
        }
    }
    true
}
