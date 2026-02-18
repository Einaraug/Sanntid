// src/orders.rs
use crate::elevio::elev::{Elevator, HALL_UP, HALL_DOWN, CAB};
use crate::elev_algo::elevator::{N_FLOORS, N_BUTTONS, Button};

// ── Order Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderState {
    None,
    Unconfirmed,
    Confirmed,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub floor: usize,
    pub button: Button,
    pub state: OrderState,
}

// ── Order Table ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OrderTable {
    pub hall: [[OrderState; 2]; N_FLOORS],  // [floor][0=up, 1=down]
    pub cab:  [OrderState; N_FLOORS],
}

impl OrderTable {
    pub fn new() -> Self {
        Self {
            hall: std::array::from_fn(|_| [OrderState::None, OrderState::None]),
            cab:  std::array::from_fn(|_| OrderState::None),
        }
    }

    pub fn set_hall(&mut self, floor: usize, button: Button, state: OrderState) {
        match button {
            Button::HallUp   => self.hall[floor][0] = state,
            Button::HallDown => self.hall[floor][1] = state,
            Button::Cab      => self.cab[floor] = state,
        }
    }

    pub fn get_hall(&self, floor: usize, button: Button) -> &OrderState {
        match button {
            Button::HallUp   => &self.hall[floor][0],
            Button::HallDown => &self.hall[floor][1],
            Button::Cab      => &self.cab[floor],
        }
    }

    pub fn clear(&mut self, floor: usize, button: Button) {
        self.set_hall(floor, button, OrderState::None);
    }

    /// Sync lights to hardware based on current order table
    pub fn update_lights(&self, hw: &Elevator) {
        for floor in 0..N_FLOORS {
            hw.call_button_light(floor as u8, HALL_UP,
                self.hall[floor][0] == OrderState::Confirmed);
            hw.call_button_light(floor as u8, HALL_DOWN,
                self.hall[floor][1] == OrderState::Confirmed);
            hw.call_button_light(floor as u8, CAB,
                self.cab[floor] == OrderState::Confirmed);
        }
    }

    /// Convert to bool requests array for use with elev_algo
    pub fn to_requests(&self) -> [[bool; N_BUTTONS]; N_FLOORS] {
        let mut requests = [[false; N_BUTTONS]; N_FLOORS];
        for floor in 0..N_FLOORS {
            requests[floor][0] = self.hall[floor][0] == OrderState::Confirmed;
            requests[floor][1] = self.hall[floor][1] == OrderState::Confirmed;
            requests[floor][2] = self.cab[floor]     == OrderState::Confirmed;
        }
        requests
    }
}
