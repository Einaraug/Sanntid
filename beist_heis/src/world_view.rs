use crate::elev_algo::elevator::{Button, Elevator, N_FLOORS};
use crate::elev_algo::fsm::ConfirmedOrder;
use crate::elev_algo::timer::Timer;
use crate::elevio::poll::ButtonEvent;
use crate::orders::*;
use crossbeam_channel as cbc;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use crate::elev_algo::elevator::{N_BUTTONS};
use crate::orders::*;
use crate::counters::*;


pub const N_NODES: usize = 3;
type ElevId = usize;


#[derive(Clone, Serialize, Deserialize)]  
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

#[derive(Clone, Serialize, Deserialize)]  
pub struct PeerAvailability{
    peer_availability: [bool; N_NODES],
    // Timer state is runtime-only and cannot be serialized (Instant isn't serializable).
    #[serde(skip)]
    peer_timer: [Timer; N_NODES],
}
impl PeerAvailability {
    pub fn new() -> Self {
        Self {
            peer_availability: [false; N_NODES], // Initially, all peers are unavailable until they announce themselves.
            peer_timer: std::array::from_fn(|_| Timer::new()),
        }
    }

    pub fn get(&self, node_id: usize) -> bool {
        self.peer_availability[node_id]
    }

    /// Set a peer's availability state.
    ///
    /// Returns `true` if the value flipped (true->false or false->true).
    pub fn set(&mut self, node_id: usize, available: bool) -> bool {
        let previous = self.peer_availability[node_id];
        if previous != available {
            self.peer_availability[node_id] = available;
            return true;
        }
        false
    }

    /// Mark that we have received a message from `node_id` just now.
    ///
    /// Returns `true` if availability flipped from false -> true.
    pub fn mark_seen(&mut self, node_id: usize, timeout: Duration) -> bool {
        let flipped = self.set(node_id, true);
        self.peer_timer[node_id].start(timeout.as_secs_f64());
        flipped
    }

    /// Returns true if `node_id` should be considered timed out.
    /// If it times out, we also mark the peer as unavailable.
    ///
    /// Returns `true` only when the peer transitions from available -> unavailable.
    pub fn should_timeout(&mut self, node_id: usize) -> bool {
        if !self.peer_availability[node_id] {
            return false; // already unavailable
        }

        if self.peer_timer[node_id].timed_out() {
            self.peer_availability[node_id] = false;
            return true;
        }

        false
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, bool)> {
        self.peer_availability.iter().copied().enumerate()
    }
}


//TODO: Generalize names
#[derive(Clone, Serialize, Deserialize)]  
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


    // Setters:
    pub fn set_elevator(&mut self, node_id: usize, elevator: Elevator) {
        if elevator != self.elevator_map.elevator[node_id] {
            self.elevator_map.set(node_id, elevator);
            self.counters.inc_elevator(node_id);   
        }
    }

    pub fn set_peer_availability(&mut self, node_id: usize, available: bool) {
        if self.peer_availability.set(node_id, available) {
            self.counters.inc_peer_availability(node_id);
        }
    }

    pub fn set_hall_order_state(&mut self, floor: usize, button: Button, state: OrderState) {
        let hall_order = self.order_table.get_hall_order_mut(floor, button as usize);

        hall_order.set_state(state);
        self.counters.inc_hall_order(floor, button);
    }

    pub fn set_hall_order_node_id(&mut self, floor: usize, button: Button, node_id: usize) {
        let hall_order = self.order_table.get_hall_order_mut(floor, button as usize);

        hall_order.set_node_id(node_id);
        self.counters.inc_hall_order(floor, button);
        }

    pub fn set_cab_order_state(&mut self, floor: usize, node_id: usize, state: OrderState) {
        let cab_order = self.order_table.get_cab_order_mut(floor, node_id);

        cab_order.set_state(state);
        self.counters.inc_cab_order(floor, node_id);
    }


    // --BUTTON PRESS HANDLER --
    pub fn on_button_press(&mut self, floor: usize, button: Button) {
        match button {
            Button::HallUp | Button::HallDown => {
                self.set_hall_order_state(floor, button, OrderState::Unconfirmed);
                self.set_hall_order_node_id(floor, button, UNASSIGNED_NODE);
            }
            Button::Cab => {
                self.set_cab_order_state(floor, self.self_id, OrderState::Unconfirmed);
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

    pub fn confirm_and_assign_orders(&mut self) {
        for floor in 0..N_FLOORS {
        // HANDLE CAB ORDERS
        for node_id in 0..N_NODES {
            let cab_order = self.order_table.get_cab_order(floor, node_id);
            let seen_by = cab_order.get_seen_by();
            if self.is_all_acked(&seen_by) {
                self.set_cab_order_state(floor, node_id,OrderState::Confirmed);
            }
        }

        // HANDLE HALL ORDERS
        for button in [Button::HallUp, Button::HallDown] {
            let hall_order = self.order_table.get_hall_order_mut(floor, button as usize);
            let seen_by = hall_order.get_seen_by();
            if self.is_all_acked(&seen_by) {
                self.set_hall_order_state(floor, button, OrderState::Confirmed);
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
    pub fn run(
        mut self,
        from_buttons: cbc::Receiver<ButtonEvent>,
        from_fsm: cbc::Receiver<Elevator>,
        from_network: cbc::Receiver<WorldView>,
        to_fsm: cbc::Sender<ConfirmedOrder>,
        to_network: cbc::Sender<WorldView>,
    ) {
        //TODO: update func names
        const BROADCAST_INTERVAL: Duration = Duration::from_millis(100);

        loop {
            cbc::select! {
                recv(from_buttons) -> msg => {
                    let Ok(btn) = msg else { break };
                    if let Some(button) = Button::from_index(btn.button as usize) {
                        self.on_button_press(btn.floor as usize, button);
                    }
                },
                recv(from_fsm) -> msg => {
                    let Ok(elev) = msg else { break };
                    self.set_elevator(self.self_id, elev);
                },
                recv(from_network) -> msg => {
                    let Ok(peer_wv) = msg else { break };
                    if peer_wv.self_id != self.self_id {
                        // Update the peer's last-seen timer and merge its worldview.
                        const PEER_TIMEOUT: Duration = Duration::from_millis(500);
                        if self.peer_availability.mark_seen(peer_wv.self_id, PEER_TIMEOUT) {
                            self.counters.inc_peer_availability(peer_wv.self_id);
                        }
                        self.merge(&peer_wv);
                    }
                },
                default(BROADCAST_INTERVAL) => {
                    // Periodic broadcast

                    // Mark peers as dead if we haven't seen them within the timeout.
                    for node in 0..N_NODES {
                        // Never time ourselves out
                        if node == self.self_id {
                            continue;
                        }
                        if self.peer_availability.should_timeout(node) {
                            self.counters.inc_peer_availability(node);
                        }
                    }
                }
            }

            // Broadcast to network
            let _ = to_network.send(self.clone());
        }
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
        let local_ct = local.counters.get_peer_availability(node);
        let incoming_ct = incoming.counters.get_peer_availability(node);

        if incoming_ct > local_ct {
            let is_available: bool = incoming_availability.get(node);

            local.peer_availability.set(node, is_available);
            local.counters.set_peer_availability(node, incoming_ct);
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
mod tests {
    use super::*;
    use crate::elev_algo::elevator::Elevator;

    #[test]
    fn set_elevator_updates_only_on_change() {
        let mut wv = WorldView::new(0);

        // Starting state: elevator is default, counter is 0
        assert_eq!(wv.get_counters().get_elevator(0), 0);

        // Setting the same elevator again should not increment the counter
        wv.set_elevator(0, Elevator::new());
        assert_eq!(wv.get_counters().get_elevator(0), 0);

        // Changing the elevator should update the stored elevator and increment the counter
        let mut new_elevator = Elevator::new();
        new_elevator.floor = 3;
        wv.set_elevator(0, new_elevator);

        assert_eq!(wv.get_counters().get_elevator(0), 1);
        assert_eq!(wv.get_elevator_map().get(0), &new_elevator);

        // Setting the same elevator again should not increment the counter further
        wv.set_elevator(0, new_elevator);
        assert_eq!(wv.get_counters().get_elevator(0), 1);
    }

    #[test]
    fn peer_availability_timeout_uses_last_seen() {
        let mut wv = WorldView::new(0);
        let peer_id = 1;

        // Mark the peer as seen right now with a short timeout.
        wv.peer_availability.mark_seen(peer_id, Duration::from_millis(10));
        assert!(wv.get_peer_availability().get(peer_id));
        assert!(!wv.peer_availability.should_timeout(peer_id));

        // Wait until the timeout is exceeded.
        std::thread::sleep(Duration::from_millis(20));
        let did_timeout = wv.peer_availability.should_timeout(peer_id);

        assert!(did_timeout, "Peer should be considered timed out");
        assert!(!wv.get_peer_availability().get(peer_id), "Peer should be marked unavailable");
    }
}
