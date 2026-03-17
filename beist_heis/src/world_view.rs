use crate::orders::OrderTable;
use crate::peer_monitor::PeerMonitor;
use crate::node_states::NodeStates;
use crate::counters::Counters;
use crate::elev_algo::elevator::{Button, N_FLOORS};
use serde::{Serialize, Deserialize};

pub const N_NODES: usize = 3;
pub const N_DIRS: usize = 2;

// Data container to be continously broadcasted
#[derive(Clone, Serialize, Deserialize)]
pub struct WorldView {
    pub self_id: usize,
    pub node_states: NodeStates,
    pub peer_monitor: PeerMonitor,
    pub order_table: OrderTable,
    pub counters: Counters,
}

impl WorldView {
    pub fn new(self_id: usize) -> Self {
        let wv = Self {
            self_id,
            node_states: NodeStates::new(),
            peer_monitor: PeerMonitor::new(self_id), 
            order_table: OrderTable::new(),
            counters: Counters::new(self_id),
        };
        wv
    }

    pub fn merge_from(&mut self, incoming: &WorldView) {
        self.merge_hall_orders(incoming);
        self.merge_cab_orders(incoming);
        self.merge_peer_availability(incoming);
        self.merge_elevators(incoming);
    }

    fn merge_hall_orders(&mut self, incoming: &WorldView) {
        for floor in 0..N_FLOORS {
            for btn in [Button::HallUp, Button::HallDown] {
                let local_ct = self.counters.get_hall_order(floor, btn);
                let incoming_ct = incoming.counters.get_hall_order(floor, btn);
                if incoming_ct > local_ct {
                    let hall_order = incoming.order_table.get_hall_order(floor, btn.to_index());
                    self.order_table.set_hall_order(floor, btn, hall_order);
                    self.order_table.set_seen_by_hall(floor, btn, self.self_id);
                    self.counters.set_hall_order(floor, btn, incoming_ct);
                } 
                else if incoming_ct == local_ct {
                    self.order_table.set_seen_by_hall(floor, btn, incoming.self_id);
                }
            }
        }
    }

    fn merge_cab_orders(&mut self, incoming: &WorldView) {
        for floor in 0..N_FLOORS {
            for node_id in 0..N_NODES {
                let local_ct = self.counters.get_cab_order(floor, node_id);
                let incoming_ct = incoming.counters.get_cab_order(floor, node_id);
                if incoming_ct > local_ct {
                    let cab_order = incoming.order_table.get_cab_order(floor, node_id);
                    self.order_table.set_cab_order(floor, node_id, cab_order);
                    self.order_table.set_seen_by_cab(floor, node_id, self.self_id);
                    self.counters.set_cab_order(floor, node_id, incoming_ct);
                } 
                else if incoming_ct == local_ct {
                    self.order_table.set_seen_by_cab(floor, node_id, incoming.self_id);
                }
            }
        }
    }

    fn merge_peer_availability(&mut self, incoming: &WorldView) {
        for node_id in 0..N_NODES {
            let local_ct = self.counters.get_peer_availability(node_id);
            let incoming_ct = incoming.counters.get_peer_availability(node_id);
            if incoming_ct > local_ct {
                let is_available = incoming.peer_monitor.availability[node_id];
                self.peer_monitor.set(node_id, is_available);
                self.counters.set_peer_availability(node_id, incoming_ct);
            }
        }
    }

    fn merge_elevators(&mut self, incoming: &WorldView) {
        for node_id in 0..N_NODES {
            let local_ct = self.counters.get_elevator(node_id);
            let incoming_ct = incoming.counters.get_elevator(node_id);
            if incoming_ct > local_ct {
                let elevator = incoming.node_states.get(node_id);
                let _ = self.node_states.set(node_id, elevator);
                self.counters.set_elevator(node_id, incoming_ct);
            }
        }
    }
}