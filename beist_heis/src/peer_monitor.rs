use crate::elev_algo::timer::Timer;
use crate::counters::Counters;
use crate::world_view::N_NODES;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Clone, Serialize, Deserialize)]
pub struct PeerMonitor {
    pub availability: [bool; N_NODES],
    #[serde(skip)]
    timers: [Timer; N_NODES],
}

impl PeerMonitor {
    pub fn new() -> Self {
        Self {
            availability: [false; N_NODES],
            timers: std::array::from_fn(|_| Timer::new()),
        }
    }

    pub fn is_available(&self, node_id: usize) -> bool {
        self.availability[node_id]
    }

    /// Called when a message from node_id is received.
    /// Increments counter if peer flipped unavailable → available.
    pub fn mark_seen(&mut self, node_id: usize, timeout: Duration, counters: &mut Counters) {
        let flipped = self.set(node_id, true);
        self.timers[node_id].start(timeout.as_secs_f64());
        if flipped {
            counters.inc_peer_availability(node_id);
        }
    }

    /// Check all nodes for timeout. Returns list of node_ids that just died.
    /// Increments counter for each transition available → unavailable.
    pub fn expire_stale_peers(&mut self, counters: &mut Counters) -> Vec<usize> {
        let mut dead = Vec::new();
        for node_id in 0..N_NODES {
            if self.availability[node_id] && self.timers[node_id].timed_out() {
                self.availability[node_id] = false;
                counters.inc_peer_availability(node_id);
                dead.push(node_id);
            }
        }
        dead
    }

    /// Set availability for a node. Returns true if value flipped.
    pub fn set(&mut self, node_id: usize, available: bool) -> bool {
        let previous = self.availability[node_id];
        if previous != available {
            self.availability[node_id] = available;
            return true;
        }
        false
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, bool)> {
        self.availability.iter().copied().enumerate()
    }
}
