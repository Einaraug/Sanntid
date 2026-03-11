use crate::elev_algo::timer::Timer;
use crate::counters::Change;
use crate::world_view::N_NODES;
use serde::{Serialize, Deserialize};
use std::time::Duration;

// If a peer misses 5 broadcast ticks, it is considered timed out.
pub const PEER_TIMEOUT: Duration = Duration::from_millis(500);

// Keeps track of and updates availability of all nodes.
#[derive(Clone, Serialize, Deserialize)]
pub struct PeerMonitor {
    pub availability: [bool; N_NODES],
    #[serde(skip)] // Availability must be serialized for assigner input and wv transmission. Timers can't and don't need to be serialized
    timers: [Timer; N_NODES],
}

impl PeerMonitor {
    pub fn new(self_id: usize) -> Self {
        let mut availability = [false; N_NODES];
        availability[self_id] = true;
        Self {
            availability,
            timers: std::array::from_fn(|_| Timer::new()),
        }
    }

    // Set availability for a node. Returns true if value flipped.
    pub fn set(&mut self, node_id: usize, availability: bool) -> bool {
        let previous_availability = self.availability[node_id];
        if previous_availability != availability {
            self.availability[node_id] = availability;
            return true;
        }
        false
    }

    pub fn is_available(&self, node_id: usize) -> bool {
        self.availability[node_id]
    }

    /// Called when a message from a peer is received. Resets the peer's timeout timer.
    pub fn mark_seen(&mut self, node_id: usize) -> Vec<Change> {
        let mut changes = Vec::new();
        let flipped = self.set(node_id, true);
        self.timers[node_id].start(PEER_TIMEOUT);
        if flipped {
            changes.push(Change::PeerAvailability { node_id });
        }
        changes
    }

    /// Check all nodes for timeout. Returns (timed-out node_ids, changes).
    pub fn expire_stale_peers(&mut self) -> (Vec<usize>, Vec<Change>) {
        let mut changes = Vec::new();
        let mut dead = Vec::new();

        for node_id in 0..N_NODES {
            if self.is_available(node_id) && self.timers[node_id].timed_out() {
                self.availability[node_id] = false;
                changes.push(Change::PeerAvailability { node_id });
                dead.push(node_id);
            }
        }
        (dead, changes)
    }
}
