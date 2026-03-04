use crate::elev_algo::elevator::{Button, Elevator, N_FLOORS};
use crate::elev_algo::fsm::ConfirmedOrder;
use crate::elevio::poll::ButtonEvent;
use crate::orders::*;
use crossbeam_channel as cbc;
use serde::{Serialize, Deserialize};
use std::time::Duration;

pub const N_NODES: usize = 3;
type ElevId = u32;


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
    pub fn get_elevator(&self, node_id: usize) -> &Elevator {
        &self.elevator[node_id]
    }
    pub fn update_elevator(&mut self, node_id: usize, elevator: Elevator) {
        self.elevator[node_id] = elevator;
    }
}

#[derive(Clone, Serialize, Deserialize)]  
pub struct PeerAvailability{
    peer_availability: [bool; N_NODES],
}
impl PeerAvailability {
    pub fn new() -> Self {
        Self {
            peer_availability: [false; N_NODES], //Initially, all peers are unavailable until they announce themselves. WorldView owns the availability, so we can initialize it here.
        }
    }
    pub fn set_availability(&mut self, node_id: usize, available: bool) {
        self.peer_availability[node_id] = available;
    }
    pub fn is_available(&self, node_id: usize) -> bool {
        self.peer_availability[node_id]
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldView {
    self_id: i32,
    elevator_map: ElevatorMap,
    peer_availability: PeerAvailability,
    order_table: OrderTable,
    counts: Counters,
}
impl WorldView {
    pub fn new(self_id: i32) -> Self {
        Self {
            self_id: self_id,
            elevator_map: ElevatorMap::new(),
            peer_availability: PeerAvailability::new(),
            order_table: OrderTable::new(),
            counts: Counters::new(),
        }
    }

    // Getters :: Should the return type be references?
    pub fn get_self_id(&self) -> i32 {
        self.self_id
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

    pub fn get_counts(&self) -> &Counters {
        &self.counts
    }

    // Setters
    pub fn update_elevator(&mut self, node_id: usize, elevator: Elevator) {
        self.elevator_map.update_elevator(node_id, elevator);
    }

    pub fn set_peer_availability(&mut self, node_id: usize, available: bool) {
        self.peer_availability.set_availability(node_id, available);
    }

    pub fn update_order_table(&mut self, floor: usize, button: Button, node_id: i32, state: OrderState) {
       self.order_table.update_hall_state(floor, button, state);
       self.order_table.update_hall_id(floor, button, node_id);
    }

    pub fn update_cab_order(&mut self, floor: usize, node_id: i32, state: OrderState) {
        self.order_table.update_cab(floor, node_id, state);
    }

    pub fn run(
        mut self,
        from_buttons: cbc::Receiver<ButtonEvent>,
        from_fsm: cbc::Receiver<Elevator>,
        from_network: cbc::Receiver<WorldView>,
        to_fsm: cbc::Sender<ConfirmedOrder>,
        to_network: cbc::Sender<WorldView>,
    ) {
        const BROADCAST_INTERVAL: Duration = Duration::from_millis(100);

        loop {
            cbc::select! {
                recv(from_buttons) -> msg => {
                    let Ok(btn) = msg else { break };
                    self.handle_button_press(btn, &to_fsm);
                },
                recv(from_fsm) -> msg => {
                    let Ok(elev) = msg else { break };
                    self.update_elevator(self.self_id as usize, elev);
                },
                recv(from_network) -> msg => {
                    let Ok(peer_wv) = msg else { break };
                    if peer_wv.self_id != self.self_id {
                        self.merge_peer(peer_wv);
                    }
                },
                default(BROADCAST_INTERVAL) => {
                    // Periodic broadcast
                }
            }

            // Broadcast to network
            let _ = to_network.send(self.clone());
        }
    }

    fn handle_button_press(&mut self, btn: ButtonEvent, to_fsm: &cbc::Sender<ConfirmedOrder>) {
        let button = Button::from_index(btn.button as usize).unwrap();

        // For now: send directly to FSM (merging/assigner TODO)
        // In full implementation: add to order table, run assigner, then send if assigned to us
        let _ = to_fsm.send(ConfirmedOrder {
            floor: btn.floor as usize,
            button,
        });
    }

    fn merge_peer(&mut self, _peer: WorldView) {
        // TODO: compare counters, merge state
    }
}





//Move this to its own module?
#[derive(Clone, Serialize, Deserialize)]
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
    pub fn inc_hall_order(&mut self, floor: usize, button: Button) {
        match button {
            Button::HallUp   => self.ct_hall_order[floor][0] += 1,
            Button::HallDown => self.ct_hall_order[floor][1] += 1,
            Button::Cab      => panic!("inc_hall_order called with Button::Cab"),
        }
    }
    pub fn inc_cab_order  (&mut self, floor: usize, node_id: usize) { self.ct_cab_order[floor][node_id] += 1; }
    pub fn inc_peer_status(&mut self, node_id: usize) { self.ct_peer_status[node_id] += 1; }
    pub fn inc_floor      (&mut self, node_id: usize) { self.ct_floor[node_id]        += 1; }
    pub fn inc_dir        (&mut self, node_id: usize) { self.ct_dir[node_id]          += 1; }
    pub fn inc_state      (&mut self, node_id: usize) { self.ct_state[node_id]        += 1; }
    pub fn inc_obstruction(&mut self, node_id: usize) { self.ct_obstruction[node_id]  += 1; }

    // ── Getters ──────────────────────────────────────────────────────────────
    pub fn get_hall_order(&self, floor: usize, button: Button) -> u64 {
        match button {
            Button::HallUp   => self.ct_hall_order[floor][0],
            Button::HallDown => self.ct_hall_order[floor][1],
            Button::Cab      => panic!("get_hall_order called with Button::Cab"),
        }
    }
    pub fn get_cab_order  (&self, floor: usize, node_id: usize) -> u64 { self.ct_cab_order[floor][node_id] }
    pub fn get_peer_status(&self, node_id: usize) -> u64 { self.ct_peer_status[node_id] }
    pub fn get_floor      (&self, node_id: usize) -> u64 { self.ct_floor[node_id]       }
    pub fn get_dir        (&self, node_id: usize) -> u64 { self.ct_dir[node_id]         }
    pub fn get_state      (&self, node_id: usize) -> u64 { self.ct_state[node_id]       }
    pub fn get_obstruction(&self, node_id: usize) -> u64 { self.ct_obstruction[node_id] }
}
