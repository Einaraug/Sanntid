use crate::elev_algo::elevator::Elevator;
use crate::orders::OrderTable;
use crate::counters::Counters;
use crate::peer_monitor::PeerMonitor;
use serde::{Serialize, Deserialize};

pub const N_NODES: usize = 3;

// ── ElevatorMap ───────────────────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
pub struct ElevatorMap {
    pub elevator: [Elevator; N_NODES],
}

impl ElevatorMap {
    pub fn new() -> Self {
        Self { elevator: [Elevator::new(); N_NODES] }
    }
    pub fn get(&self, node_id: usize) -> &Elevator {
        &self.elevator[node_id]
    }
    pub fn set(&mut self, node_id: usize, elevator: Elevator) {
        self.elevator[node_id] = elevator;
    }
}

// ── WorldView (pure data container) ──────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldView {
    pub self_id:      usize,
    pub elevator_map: ElevatorMap,
    pub peer_monitor: PeerMonitor,
    pub order_table:  OrderTable,
    pub counters:     Counters,
}

impl WorldView {
    pub fn new(self_id: usize) -> Self {
        Self {
            self_id,
            elevator_map: ElevatorMap::new(),
            peer_monitor: PeerMonitor::new(),
            order_table:  OrderTable::new(),
            counters:     Counters::new(),
        }
    }
}
