use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::world_view::N_NODES;


#[derive(Clone)]
pub struct Counters{
    ct_hall_order:  [[u64; 2]; N_FLOORS],
    ct_cab_order:   [[u64; N_NODES]; N_FLOORS],
    ct_peer_status: [u64; N_NODES],
    ct_floor:       [u64; N_NODES],
    ct_dir:         [u64; N_NODES],
    ct_state:       [u64; N_NODES],
    ct_obstruction: [u64; N_NODES],
}
impl Counters{
    pub fn new() -> Self {
        Self {
            ct_hall_order:  [[0; 2]; N_FLOORS],
            ct_cab_order:   [[0; N_NODES]; N_FLOORS],
            ct_peer_status: [0; N_NODES],
            ct_floor:       [0; N_NODES],
            ct_dir:         [0; N_NODES],
            ct_state:       [0; N_NODES],
            ct_obstruction: [0; N_NODES],
        }
    }

    // ── Increment ────────────────────────────────────────────────────────────
    pub fn inc_ct_hall_order(&mut self, floor: usize, button: Button) {
        match button {
            Button::HallUp   => self.ct_hall_order[floor][0] += 1,
            Button::HallDown => self.ct_hall_order[floor][1] += 1,
            Button::Cab      => panic!("inc_hall_order called with Button::Cab"),
        }
    }
    pub fn inc_ct_cab_order  (&mut self, floor: usize, node_id: usize) { self.ct_cab_order[floor][node_id] += 1; }
    pub fn inc_ct_peer_status(&mut self, node_id: usize) { self.ct_peer_status[node_id] += 1; }
    pub fn inc_ct_floor      (&mut self, node_id: usize) { self.ct_floor[node_id]       += 1; }
    pub fn inc_ct_dir        (&mut self, node_id: usize) { self.ct_dir[node_id]         += 1; }
    pub fn inc_ct_state      (&mut self, node_id: usize) { self.ct_state[node_id]       += 1; }
    pub fn inc_ct_obstruction(&mut self, node_id: usize) { self.ct_obstruction[node_id] += 1; }

    // ── Getters ──────────────────────────────────────────────────────────────
    pub fn get_ct_hall_order(&self, floor: usize, button: Button) -> u64 {
        match button {
            Button::HallUp   => self.ct_hall_order[floor][0],
            Button::HallDown => self.ct_hall_order[floor][1],
            Button::Cab      => panic!("get_hall_order called with Button::Cab"),
        }
    }
    pub fn get_ct_cab_order  (&self, floor: usize, node_id: usize) -> u64 { self.ct_cab_order[floor][node_id] }
    pub fn get_ct_peer_status(&self, node_id: usize) -> u64 { self.ct_peer_status[node_id] }
    pub fn get_ct_floor      (&self, node_id: usize) -> u64 { self.ct_floor[node_id]       }
    pub fn get_ct_dir        (&self, node_id: usize) -> u64 { self.ct_dir[node_id]         }
    pub fn get_ct_state      (&self, node_id: usize) -> u64 { self.ct_state[node_id]       }
    pub fn get_ct_obstruction(&self, node_id: usize) -> u64 { self.ct_obstruction[node_id] }

    // ── Setters ──────────────────────────────────────────────────────────────
    pub fn set_hall_order(&mut self, floor: usize, button: Button, value: u64) {
        match button {
            Button::HallUp   => self.ct_hall_order[floor][0] = value,
            Button::HallDown => self.ct_hall_order[floor][1] = value,
            Button::Cab      => panic!("set_hall_order called with Button::Cab"),
        }
    }

    pub fn set_ct_cab_order  (&mut self, floor: usize, node_id: usize, value: u64) { self.ct_cab_order[floor][node_id] = value; }
    pub fn set_ct_peer_status(&mut self, node_id: usize, value: u64) { self.ct_peer_status[node_id] = value; }
    pub fn set_ct_floor      (&mut self, node_id: usize, value: u64) { self.ct_floor[node_id]       = value; }
    pub fn set_ct_dir        (&mut self, node_id: usize, value: u64) { self.ct_dir[node_id]         = value; }
    pub fn set_ct_state      (&mut self, node_id: usize, value: u64) { self.ct_state[node_id]       = value; }
    pub fn set_ct_obstruction(&mut self, node_id: usize, value: u64) { self.ct_obstruction[node_id] = value; }
}



fn merge worldviews(&mut my_world_view: WorldView, &received_world_view: Worldview){
    merge hall orders
    merge cab orders
    merge peer status
    merge floor
    merge dir
    merge state
    merge obstruction
}

fn merge_hall_orders(my_world_view: &mut WorldView, received_world_view: &WorldView){
    for floor in 0..N_FLOORS{
        for button in [Button::HallUp, Button::HallDown]{
            let local_ct = my_world_view.counts.get_ct_hall_order(floor, button);
            let incoming_ct = received_world_view.counts.get_ct_hall_order(floor, button);
            if incoming_ct > local_ct {
                let incoming_state = received_world_view.order_table.get_hall_state(floor, button);
                my_world_view.order_table.update_hall(floor, button, incoming_state);
                my_world_view.counts.set_ct_hall_order(floor, button, incoming_ct);
            } //if
        } //for button
    } //for floor
} //fn

fn merge_cab_orders(my_world_view: &mut WorldView, received_world_view: &WorldView) {
    for floor in 0..N_FLOORS {
        for node in 0..N_NODES {
            let local_ct = my_world_view.counts.get_ct_cab_order(floor, node);
            let incoming_ct = received_world_view.counts.get_ct_cab_order(floor, node);
            if incoming_ct > local_ct {
                let incoming_state = received_world_view.order_table.get_cab_state(floor, node);
                my_world_view.order_table.update_cab(floor, node, incoming_state);
                my_world_view.counts.set_ct_cab_order(floor, received_world_view.self_id as usize, incoming_ct);
            }
        }
    }
}

fn merge_peer_status(my_world_view: &mut WorldView, received_world_view: &WorldView) {
    let me = my_world_view.self_id;
    let incoming_node = received_world_view.self_id
    let local_ct = my_world_view.counts.get_ct_peer_status(me);
    let incoming_ct = received_world_view.counts.get_ct_peer_status(incoming_node);
    if incoming_ct > local_ct {
        let my_world_view.peer_availability = received_world_view.peer_availability;
        my_world_view.counts.set_ct_peer_status(received_world_view.self_id, incoming_ct);
    }
}