use crate::elevio::elev as hw;
use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::orders::{OrderTable, OrderState};
use crossbeam_channel as cbc;

pub fn run(hw: hw::Elevator, self_id: usize, rx: cbc::Receiver<OrderTable>) {
    for order_table in rx {
        for floor in 0..N_FLOORS {
            for btn in [Button::HallUp, Button::HallDown] {
                let on = order_table.get_hall_order(floor, btn.to_index()).state == OrderState::Confirmed;
                hw.call_button_light(floor as u8, btn.to_index() as u8, on);
            }
            let on = order_table.get_cab_order(floor, self_id).state == OrderState::Confirmed;
            hw.call_button_light(floor as u8, Button::Cab.to_index() as u8, on);
        }
    }
}
