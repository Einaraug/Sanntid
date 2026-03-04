use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::world_view::N_NODES;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderState {
    None,
    Unconfirmed,
    Confirmed,
}

#[derive(Debug, Clone, Copy)]
pub struct HallOrder{
    pub state: OrderState,
    pub id: i32,
}

impl HallOrder{
    pub fn new() -> Self {
        Self{
            state: OrderState::None,
            id: 0, //0 means no node owns the order
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderTable {
    pub hall: [[HallOrder; 2]; N_FLOORS],
    pub cab:  [[OrderState; N_NODES]; N_FLOORS],
}

impl OrderTable {
    pub fn new() -> Self {
        Self {
            hall: [[HallOrder::new(); 2]; N_FLOORS],
            cab:  [[OrderState::None; N_NODES]; N_FLOORS],
        }
    }

    pub fn update_hall_state(&mut self, floor: usize, button: Button, state: OrderState) {
        match button {
            Button::HallUp   => self.hall[floor][0].state = state,
            Button::HallDown => self.hall[floor][1].state = state,
            Button::Cab      => panic!("update_hall called with Button::Cab"),
        }
    }

    pub fn update_hall_id(&mut self, floor: usize, button: Button, id: i32) {
        match button {
            Button::HallUp   => self.hall[floor][0].id = id,
            Button::HallDown => self.hall[floor][1].id = id,
            Button::Cab      => panic!("update_hall called with Button::Cab"),
        }
    }

    pub fn update_cab(&mut self, floor: usize, node_id: i32, state: OrderState) {
        self.cab[floor][node_id] = state;
    }
   
    pub fn get_hall_state(&self, floor: usize, button: Button) -> OrderState {
    match button {
        Button::HallUp   => self.hall[floor][0].state,
        Button::HallDown => self.hall[floor][1].state,
        Button::Cab      => panic!("Use get_cab_state for cab orders"),
        }
    }   

    pub fn get_cab_state(&self, floor: usize, node_id: i32) -> OrderState {
        self.cab[floor][node_id]
    }

    pub fn clear_hall(&mut self, floor: usize, button: Button) {
        self.update_hall_state(floor, button, OrderState::None);
        self.update_hall_id(floor, button, 0);
    }

    pub fn clear_cab(&mut self, floor: usize, node_id: i32) {
        self.update_cab(floor, node_id, OrderState::None);
    }

    pub fn on_button_press(&mut self, floor: usize, button: Button, node_id: i32) {
        match button {
            Button::HallUp | Button::HallDown => {
                self.update_hall_state(floor, button, OrderState::Unconfirmed);
                self.update_hall_id(floor, button, 0); //0 indicates unassigned order
            }
            Button::Cab => {
                self.update_cab(floor, node_id, OrderState::Unconfirmed);
            }
        }
    }
}
 