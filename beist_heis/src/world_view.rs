#![allow(dead_code, non_snake_case)]
use crate::elev_algo::elevator::{Button, Elevator, N_BUTTONS, N_FLOORS};
use crate::orders::*;
use crate::counters::*;


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
    pub fn get(&self, node_id: usize) -> &Elevator {
        &self.elevator[node_id]
    }
    pub fn set(&mut self, node_id: usize, elevator: Elevator) {
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
    pub fn get(&self, node_id: usize) -> bool {
        self.peer_availability[node_id]
    }
    pub fn set(&mut self, node_id: usize, available: bool) {
        self.peer_availability[node_id] = available;
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, bool)> {
        self.peer_availability.iter().copied().enumerate()
    }
}
//TODO: Generalize names
pub struct WorldView {
    self_id: usize,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counters: Counters,
}
impl WorldView {
    pub fn new(self_id: usize) -> Self {
        Self {
            self_id: self_id,
            elevator_map: ElevatorMap::new(),
            peer_availability: PeerAvailability::new(),
            order_table: OrderTable::new(),
            counters: Counters::new(),
        }
    }

    // Getters :: Should the return type be references?
    pub fn get_self_id(&self) -> &usize {
        &self.self_id
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

    pub fn get_counters(&self) -> &Counters {
        &self.counters
    }

    // --BUTTON PRESS HANDLER --
    pub fn on_button_press(&mut self, floor: usize, button: Button) {
        match button {
            Button::HallUp | Button::HallDown => {
                let order = self.order_table.get_hall_order_mut(floor, button as usize);
                order.set_state(OrderState::Unconfirmed);
                order.set_node_id(UNASSIGNED_NODE);
                order.set_seen_by(self.self_id);
            }
            Button::Cab => {
                let order = self.order_table.get_cab_order_mut(floor, self.self_id);
                order.set_state(OrderState::Unconfirmed);
                order.set_seen_by(self.self_id);
            }
        }
    }

    pub fn is_all_acked(&self, seen_by: &[bool; N_NODES]) -> bool {
        for (node_id, available) in self.peer_availability.iter() {
            if available && !seen_by[node_id] {
                return false;
            }
        }
        true
    }
    pub fn modify_order_states(&mut self) {
        for floor in 0..N_FLOORS {
        // HANDLE CAB ORDERS
        for node_id in 0..N_NODES {
            let seen_by = self.order_table.get_cab_order(floor, node_id).get_seen_by();
            if self.is_all_acked(&seen_by) {
                self.order_table.get_cab_order_mut(floor, node_id).set_state(OrderState::Confirmed);
            }
        }

        // HANDLE HALL ORDERS
        for button in [Button::HallUp, Button::HallDown] {
            let seen_by = self.order_table.get_hall_order(floor, button as usize).get_seen_by();
            if self.is_all_acked(&seen_by) {
                self.order_table.get_hall_order_mut(floor, button as usize).set_state(OrderState::Confirmed);
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
    pub fn merge(&mut self, incoming: &WorldView){
        //Should be atomic?
        merge_hall_orders(self, incoming);
        merge_cab_orders(self, incoming);
        merge_peer_status(self, incoming);
        merge_elevator(self, incoming);
    }
}


//Helper functions for merging worldviews
fn merge_hall_orders(local: &mut WorldView, incoming: &WorldView){
    for floor in 0..N_FLOORS{
        for button in [Button::HallUp, Button::HallDown]{
            let local_ct = local.counters.get_hall_order(floor, button);
            let incoming_ct = incoming.counters.get_hall_order(floor, button);
            let local_order = local.order_table.get_hall_order_mut(floor, button as usize);

            //Incoming world_view with more recent state
            if incoming_ct > local_ct {
                let incoming_order = incoming.order_table.get_hall_order(floor, button as usize);
                *local_order = incoming_order;
                local.counters.set_hall_order(floor, button, incoming_ct);
            }

            else if incoming_ct == local_ct {
                let incoming_id = incoming.self_id;
                local_order.set_seen_by(incoming_id);
            }
        }
    }
}

fn merge_cab_orders(local: &mut WorldView, incoming: &WorldView) {
    for floor in 0..N_FLOORS {
        for node_id in 0..N_NODES {
            let local_ct = local.counters.get_cab_order(floor, node_id);
            let incoming_ct = incoming.counters.get_cab_order(floor, node_id);
            let local_order = local.order_table.get_cab_order_mut(floor, node_id);

            if incoming_ct > local_ct {
                let incoming_order = incoming.order_table.get_cab_order(floor, node_id);
                *local_order = incoming_order;
                local.counters.set_cab_order(floor, node_id, incoming_ct);
            }
            else if incoming_ct == local_ct {
                let incoming_id = incoming.self_id;
                local_order.set_seen_by(incoming_id);
            }
        }
    }
}

fn merge_peer_status(local: &mut WorldView, incoming: &WorldView) {
    let incoming_availability = incoming.get_peer_availability();
    for node in 0..N_NODES {
        let local_ct = local.counters.get_peer_status(node);
        let incoming_ct = incoming.counters.get_peer_status(node);

        if incoming_ct > local_ct {
            let is_available: bool = incoming_availability.get(node);

            local.peer_availability.set(node, is_available);
            local.counters.set_peer_status(node, incoming_ct);
        }
    }
}

fn merge_elevator(local: &mut WorldView, incoming: &WorldView) {
    for node in 0..N_NODES {
        let local_ct = local.counters.get_elevator(node);
        let incoming_ct = incoming.counters.get_elevator(node);
        if incoming_ct > local_ct {
            let incoming_elevator = *incoming.elevator_map.get(node);
            local.elevator_map.set(node, incoming_elevator);
            local.counters.set_elevator(node, incoming_ct);
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::elev_algo::elevator::Button;
    use crate::elev_algo::elevator::Dirn;

    fn gen_wv(id: usize) ->  WorldView{
        let mut wv = WorldView::new(id);
        for i in 0..N_NODES{
            wv.peer_availability.set(i, true);
        }
        wv
    }

    #[test]
    fn try_to_add_order(){
        let mut wv = gen_wv(0);
        wv.on_button_press(1, Button::HallUp);
        wv.on_button_press(2, Button::HallDown);
        wv.on_button_press(2, Button::Cab);

        let cabOrder_2_correct = wv.order_table.get_cab_order(2, 0);
        let cabOrder_2_incorrect= wv.order_table.get_cab_order(2, 1);
                
        assert_eq!(cabOrder_2_correct.get_state(), OrderState::Unconfirmed);
        assert_eq!(cabOrder_2_incorrect.get_state(), OrderState::None);

        let hallOrder_1_up = wv.order_table.get_hall_order(1, Button::HallUp as usize);
        let  hallOrder_2_down = wv.order_table.get_hall_order_mut(2, Button::HallDown as usize);


        assert_eq!(hallOrder_1_up.get_state(), OrderState::Unconfirmed);
        assert_eq!(hallOrder_1_up.get_node_id(), UNASSIGNED_NODE);

        assert_eq!(hallOrder_2_down.get_state(), OrderState::Unconfirmed);
        assert_eq!(hallOrder_2_down.get_node_id(), UNASSIGNED_NODE);
        hallOrder_2_down.set_node_id(2);
        assert_eq!(hallOrder_2_down.get_node_id(), 2);
    }
    #[test]
    fn test_seen_by() {
        let mut wv = gen_wv(0);
        wv.on_button_press(1, Button::HallUp);
        wv.on_button_press(2, Button::HallDown);

        let one_up = wv.order_table.get_hall_order_mut(1, Button::HallUp as usize);
        one_up.set_seen_by(1);
        one_up.set_seen_by(2);
        
        assert_eq!(one_up.get_seen_by(), [true, true, true]);

        let two_down = wv.order_table.get_hall_order_mut(2, Button::HallDown as usize);
        two_down.set_seen_by(2);
        two_down.set_seen_by(1);

        wv.modify_order_states();

        assert_eq!(
            wv.order_table.get_hall_order(1, Button::HallUp as usize).get_state(),
            OrderState::Confirmed
        );
        assert_eq!(wv.order_table.get_hall_order(2, Button::HallDown as usize).get_state(), OrderState::Confirmed);
    }

    // --- Merge helper function tests -----------------------------------------------------

    #[test]
    fn merge_helpers_hall_orders() {
        let mut local = gen_wv(0);
        let mut incoming = gen_wv(1);

        // incoming creates a new hall order and bumps its counter
        incoming.on_button_press(2, Button::HallDown);
        incoming.counters.inc_hall_order(2, Button::HallDown);
        assert_eq!(incoming.order_table.get_hall_order(2, Button::HallDown as usize).get_state(), OrderState::Unconfirmed);

        // first merge: local is outdated and should adopt the incoming order and counter
        merge_hall_orders(&mut local, &incoming);
        let local_order = local.order_table.get_hall_order(2, Button::HallDown as usize);
        assert_eq!(local_order.get_state(), OrderState::Unconfirmed);
        assert_eq!(local.counters.get_hall_order(2, Button::HallDown), incoming.counters.get_hall_order(2, Button::HallDown));

        // equal counters should add incoming id to seen_by
        merge_hall_orders(&mut local, &incoming);
        let seen_by = local.order_table.get_hall_order(2, Button::HallDown as usize).get_seen_by();
        assert_eq!(seen_by[incoming.self_id], true);

        // if incoming count drops below local, nothing should change
        incoming.counters.set_hall_order(2, Button::HallDown, 0);
        incoming.order_table.get_hall_order_mut(2, Button::HallDown as usize).set_state(OrderState::None);
        merge_hall_orders(&mut local, &incoming);
        assert_eq!(local.order_table.get_hall_order(2, Button::HallDown as usize).get_state(), OrderState::Unconfirmed);
    }

    #[test]
    fn merge_helpers_cab_orders() {
        let mut local = gen_wv(0);
        let mut incoming = gen_wv(1);

        // incoming creates a cab order for its own node and bumps its counter
        incoming.on_button_press(1, Button::Cab);
        incoming.counters.inc_cab_order(1, incoming.self_id);
        assert_eq!(incoming.order_table.get_cab_order(1, incoming.self_id).get_state(), OrderState::Unconfirmed);

        merge_cab_orders(&mut local, &incoming);
        let local_cab = local.order_table.get_cab_order(1, incoming.self_id);
        assert_eq!(local_cab.get_state(), OrderState::Unconfirmed);
        assert_eq!(local.counters.get_cab_order(1, incoming.self_id), incoming.counters.get_cab_order(1, incoming.self_id));

        // equal counters should set seen_by
        merge_cab_orders(&mut local, &incoming);
        let seen = local.order_table.get_cab_order(1, incoming.self_id).get_seen_by();
        assert!(seen[incoming.self_id]);

        // simulate second update: incoming increments again and moves to Confirmed
        incoming.counters.inc_cab_order(1, incoming.self_id);
        incoming.order_table.get_cab_order_mut(1, incoming.self_id).set_state(OrderState::Confirmed);
        merge_cab_orders(&mut local, &incoming);
        // after this merge local should adopt the new state and higher counter
        assert_eq!(local.order_table.get_cab_order(1, incoming.self_id).get_state(), OrderState::Confirmed);
        assert_eq!(local.counters.get_cab_order(1, incoming.self_id), incoming.counters.get_cab_order(1, incoming.self_id));

        // incoming behind: lower counter shouldn't overwrite local
        incoming.counters.set_cab_order(1, incoming.self_id, 0);
        incoming.order_table.get_cab_order_mut(1, incoming.self_id).set_state(OrderState::None);
        merge_cab_orders(&mut local, &incoming);
        assert_eq!(local.order_table.get_cab_order(1, incoming.self_id).get_state(), OrderState::Confirmed);
    }

    #[test]
    fn merge_helpers_peer_status() {
        let mut local = gen_wv(0);
        let mut incoming = gen_wv(1);

        // incoming announces node 2 available and increments its counter
        incoming.peer_availability.set(2, true);
        incoming.counters.inc_peer_status(2);
        merge_peer_status(&mut local, &incoming);

        assert!(local.peer_availability.get(2));
        assert_eq!(local.counters.get_peer_status(2), incoming.counters.get_peer_status(2));

        // equal counters: availability should remain unchanged
        merge_peer_status(&mut local, &incoming);
        assert!(local.peer_availability.get(2));
    }

    #[test]
    fn merge_helpers_elevator() {
        let mut local = gen_wv(0);
        let mut incoming = gen_wv(1);

        // modify incoming elevator state and bump counter
        let mut elev = Elevator::new();
        elev.floor = 2;
        elev.dirn = Dirn::Up;
        incoming.elevator_map.set(1, elev);
        incoming.counters.inc_elevator(1);

        merge_elevator(&mut local, &incoming);
        assert_eq!(local.elevator_map.get(1).floor, 2);
        assert_eq!(local.counters.get_elevator(1), 1);

        // equal count should not override or change anything
        merge_elevator(&mut local, &incoming);
        assert_eq!(local.elevator_map.get(1).floor, 2);
    }

    #[test]
    fn merge_worldview_function() {
        let mut local = gen_wv(0);
        let mut incoming = gen_wv(1);

        // prepare incoming with a hall order, cab order, peer status, and elevator state
        incoming.on_button_press(0, Button::HallUp);
        incoming.counters.inc_hall_order(0, Button::HallUp);

        incoming.on_button_press(1, Button::Cab);
        incoming.counters.inc_cab_order(1, incoming.self_id);

        incoming.peer_availability.set(2, false);
        incoming.counters.inc_peer_status(2);

        let mut elev = Elevator::new();
        elev.floor = 3;
        elev.dirn = Dirn::Down;
        incoming.elevator_map.set(2, elev);
        incoming.counters.inc_elevator(2);

        // merge via the public method
        local.merge(&incoming);

        // verify all components were merged
        assert_eq!(local.order_table.get_hall_order(0, Button::HallUp as usize).get_state(), OrderState::Unconfirmed);
        assert_eq!(local.counters.get_hall_order(0, Button::HallUp), incoming.counters.get_hall_order(0, Button::HallUp));

        assert_eq!(local.order_table.get_cab_order(1, incoming.self_id).get_state(), OrderState::Unconfirmed);
        assert_eq!(local.counters.get_cab_order(1, incoming.self_id), incoming.counters.get_cab_order(1, incoming.self_id));

        assert!(!local.peer_availability.get(2));
        assert_eq!(local.counters.get_peer_status(2), incoming.counters.get_peer_status(2));

        assert_eq!(local.elevator_map.get(2).floor, 3);
        assert_eq!(local.counters.get_elevator(2), incoming.counters.get_elevator(2));

        // second merge with equal counters should only update seen_by for hall and cab
        local.merge(&incoming);
        assert!(local.order_table.get_hall_order(0, Button::HallUp as usize).get_seen_by()[incoming.self_id]);
        assert!(local.order_table.get_cab_order(1, incoming.self_id).get_seen_by()[incoming.self_id]);
    }
    #[test]
    fn merge_worldviews_bidirectional() {
        let mut local = gen_wv(0);
        let mut incoming = gen_wv(1);

        // local updates
        local.on_button_press(0, Button::HallUp);
        local.counters.inc_hall_order(0, Button::HallUp);

        // local cab order for its own node with higher counter
        local.on_button_press(1, Button::Cab);
        local.counters.inc_cab_order(1, local.self_id);
        local.counters.inc_cab_order(1, local.self_id); // counter = 2
        
        // local peer status node 2 true counter1
        local.peer_availability.set(2, true);
        local.counters.inc_peer_status(2);
        // local elevator node1 floor1 counter1
        let mut elev = Elevator::new();
        elev.floor = 1;
        elev.dirn = Dirn::Up;
        local.elevator_map.set(1, elev);
        local.counters.inc_elevator(1);

        // incoming updates
        incoming.on_button_press(0, Button::HallUp);
        incoming.counters.inc_hall_order(0, Button::HallUp);
        // incoming cab order for its own node with counter1 (same floor as local)
        incoming.on_button_press(1, Button::Cab);
        incoming.counters.inc_cab_order(1, incoming.self_id);
        // incoming peer status node 2 false counter2
        incoming.peer_availability.set(2, false);
        incoming.counters.inc_peer_status(2);
        incoming.counters.inc_peer_status(2);
        // incoming elevator node1 floor2 counter2
        let mut elev2 = Elevator::new();
        elev2.floor = 2;
        elev2.dirn = Dirn::Down;
        incoming.elevator_map.set(1, elev2);
        incoming.counters.inc_elevator(1);
        incoming.counters.inc_elevator(1);

        // perform merge with local being merged by incoming
        local.merge(&incoming);

        // Hall order counter equal, seen_by should include both ids and state unconfirmed
        let hall = local.order_table.get_hall_order(0, Button::HallUp as usize);
        assert_eq!(hall.get_state(), OrderState::Unconfirmed);
        assert!(hall.get_seen_by()[local.self_id]);
        assert!(hall.get_seen_by()[incoming.self_id]);
        assert_eq!(local.counters.get_hall_order(0, Button::HallUp), 1);

        // Cab orders: local's counter 2 > incoming's 1, so local should keep its order state
        let cab_local = local.order_table.get_cab_order(1, local.self_id);
        assert_eq!(cab_local.get_state(), OrderState::Unconfirmed);
        assert_eq!(local.counters.get_cab_order(1, local.self_id), 2);

        // incoming node's cab order should be added with counter1
        let cab_incoming = local.order_table.get_cab_order(1, incoming.self_id);
        assert_eq!(cab_incoming.get_state(), OrderState::Unconfirmed);
        assert_eq!(local.counters.get_cab_order(1, incoming.self_id), 1);

        // Peer status: incoming counter2 > local1 -> local availability should become false
        assert!(!local.peer_availability.get(2));
        assert_eq!(local.counters.get_peer_status(2), 2);

        // Elevator: incoming counter2 > local1 -> local elevator updated to incoming state
        assert_eq!(local.elevator_map.get(1).floor, 2);
        assert_eq!(local.counters.get_elevator(1), 2);
    }
}