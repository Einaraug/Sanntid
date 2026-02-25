use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::world_view::N_NODES;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderState {
    None,
    Unconfirmed,
    Confirmed,
}

pub struct HallOrder{
    pub state: OrderState,
    pub id: i32,
}

set_hall_requests(hall, cab)[
    hallRequests[:][:] = hall[if OrderState == Confirmed && order.id = None]
        hall[i][j] = Assigned
]


#[derive(Debug, Clone)]
pub struct OrderTable {
    pub hall: [[OrderState; 2]; N_FLOORS],
    pub cab:  [[OrderState; N_NODES]; N_FLOORS],
}

impl OrderTable {
    pub fn new() -> Self {
        Self {
            hall: [[OrderState::None; 2]; N_FLOORS],
            cab:  [[OrderState::None; N_NODES]; N_FLOORS],
        }
    }

    pub fn update_hall(&mut self, floor: usize, button: Button, state: OrderState) {
        match button {
            Button::HallUp   => self.hall[floor][0] = state,
            Button::HallDown => self.hall[floor][1] = state,
            Button::Cab      => panic!("update_hall called with Button::Cab"),
        }
    }

    pub fn update_cab(&mut self, floor: usize, node_id: usize, state: OrderState) {
        self.cab[floor][node_id] = state;
    }
   
    pub fn get_hall_state(&self, floor: usize, button: Button) -> OrderState {
    match button {
        Button::HallUp   => self.hall[floor][0],
        Button::HallDown => self.hall[floor][1],
        Button::Cab      => panic!("Use get_cab_state for cab orders"),
        }
    }   

    pub fn get_cab_state(&self, floor: usize, node_id: usize) -> OrderState {
        self.cab[floor][node_id]
    }

    pub fn clear_hall(&mut self, floor: usize, button: Button) {
        self.update_hall(floor, button, OrderState::None);
    }

    pub fn clear_cab(&mut self, floor: usize, node_id: usize) {
        self.update_cab(floor, node_id, OrderState::None);
    }
}
