use crate::elev_algo::elevator::Elevator;
use crate::orders::OrderTable;
use crate::nodes::Nodes;
use crate::counters::{Counters, Change};
use crate::peer_monitor::PeerMonitor;
use serde::{Serialize, Deserialize};

pub const N_NODES: usize = 3;
pub const N_DIRS: usize = 2;


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
        let mut wv = Self {
            self_id,
            elevator_map: ElevatorMap::new(),
            peer_monitor: PeerMonitor::new(),
            order_table:  OrderTable::new(),
            counters:     Counters::new(),
        };
        wv.peer_monitor.availability[self_id] = true;
        wv.counters.inc_peer_availability(self_id);
        wv
    }
}
