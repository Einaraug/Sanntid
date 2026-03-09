use serde::{Serialize, Deserialize};
use crate::elev_algo::elevator::{Button, N_FLOORS};
use crate::world_view::{WorldView, N_NODES};

pub enum Change {
    HallOrder { floor: usize, button: Button },
    CabOrder  { floor: usize, node_id: usize },
    Elevator  { node_id: usize },
    PeerAvail { node_id: usize },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Counters {
    hall_order:        [[u64; 2]; N_FLOORS],
    cab_order:         [[u64; N_NODES]; N_FLOORS],
    peer_availability: [u64; N_NODES],
    elevator:          [u64; N_NODES],
}

impl Counters {
    pub fn new() -> Self {
        Self {
            hall_order:        [[0; 2]; N_FLOORS],
            cab_order:         [[0; N_NODES]; N_FLOORS],
            peer_availability: [0; N_NODES],
            elevator:          [0; N_NODES],
        }
    }

    // ── Increment ─────────────────────────────────────────────────────────────
    pub fn inc_hall_order(&mut self, floor: usize, button: Button) {
        self.hall_order[floor][button.to_index()] += 1;
    }
    pub fn inc_cab_order(&mut self, floor: usize, node_id: usize) {
        self.cab_order[floor][node_id] += 1;
    }
    pub fn inc_peer_availability(&mut self, node_id: usize) {
        self.peer_availability[node_id] += 1;
    }
    pub fn inc_elevator(&mut self, node_id: usize) {
        self.elevator[node_id] += 1;
    }

    // ── Getters ───────────────────────────────────────────────────────────────
    pub fn get_hall_order(&self, floor: usize, button: Button) -> u64 {
        self.hall_order[floor][button.to_index()]
    }
    pub fn get_cab_order(&self, floor: usize, node_id: usize) -> u64 {
        self.cab_order[floor][node_id]
    }
    pub fn get_peer_availability(&self, node_id: usize) -> u64 {
        self.peer_availability[node_id]
    }
    pub fn get_elevator(&self, node_id: usize) -> u64 {
        self.elevator[node_id]
    }

    // ── Setters ───────────────────────────────────────────────────────────────
    pub fn set_hall_order(&mut self, floor: usize, button: Button, value: u64) {
        self.hall_order[floor][button.to_index()] = value;
    }
    pub fn set_cab_order(&mut self, floor: usize, node_id: usize, value: u64) {
        self.cab_order[floor][node_id] = value;
    }
    pub fn set_peer_availability(&mut self, node_id: usize, value: u64) {
        self.peer_availability[node_id] = value;
    }
    pub fn set_elevator(&mut self, node_id: usize, value: u64) {
        self.elevator[node_id] = value;
    }

    pub fn apply(&mut self, changes: Vec<Change>) {
        for change in changes {
            match change {
                Change::HallOrder { floor, button }  => self.inc_hall_order(floor, button),
                Change::CabOrder  { floor, node_id } => self.inc_cab_order(floor, node_id),
                Change::Elevator  { node_id }        => self.inc_elevator(node_id),
                Change::PeerAvail { node_id }        => self.inc_peer_availability(node_id),
            }
        }
    }
}

// ── Merge protocol (free functions) ──────────────────────────────────────────

pub fn merge(local: &mut WorldView, incoming: &WorldView) {
    merge_hall_orders(local, incoming);
    merge_cab_orders(local, incoming);
    merge_peer_availability(local, incoming);
    merge_elevators(local, incoming);
}

fn merge_hall_orders(local: &mut WorldView, incoming: &WorldView) {
    for floor in 0..N_FLOORS {
        for btn in [Button::HallUp, Button::HallDown] {
            let local_ct    = local.counters.get_hall_order(floor, btn);
            let incoming_ct = incoming.counters.get_hall_order(floor, btn);

            if incoming_ct > local_ct {
                let order = incoming.order_table.get_hall_order(floor, btn.to_index());
                local.order_table.replace_hall_order(floor, btn, order);
                local.counters.set_hall_order(floor, btn, incoming_ct);
            } else if incoming_ct == local_ct {
                local.order_table.set_seen_by_hall(floor, btn, incoming.self_id);
            }
        }
    }
}

fn merge_cab_orders(local: &mut WorldView, incoming: &WorldView) {
    for floor in 0..N_FLOORS {
        for node_id in 0..N_NODES {
            let local_ct    = local.counters.get_cab_order(floor, node_id);
            let incoming_ct = incoming.counters.get_cab_order(floor, node_id);

            if incoming_ct > local_ct {
                let order = incoming.order_table.get_cab_order(floor, node_id);
                local.order_table.replace_cab_order(floor, node_id, order);
                local.counters.set_cab_order(floor, node_id, incoming_ct);
            } else if incoming_ct == local_ct {
                local.order_table.set_seen_by_cab(floor, node_id, incoming.self_id);
            }
        }
    }
}

fn merge_peer_availability(local: &mut WorldView, incoming: &WorldView) {
    for node in 0..N_NODES {
        let local_ct    = local.counters.get_peer_availability(node);
        let incoming_ct = incoming.counters.get_peer_availability(node);

        if incoming_ct > local_ct {
            let available = incoming.peer_monitor.availability[node];
            local.peer_monitor.set(node, available);
            local.counters.set_peer_availability(node, incoming_ct);
        }
    }
}

fn merge_elevators(local: &mut WorldView, incoming: &WorldView) {
    for node in 0..N_NODES {
        let local_ct    = local.counters.get_elevator(node);
        let incoming_ct = incoming.counters.get_elevator(node);

        if incoming_ct > local_ct {
            let elev = *incoming.elevator_map.get(node);
            local.elevator_map.set(node, elev);
            local.counters.set_elevator(node, incoming_ct);
        }
    }
}
