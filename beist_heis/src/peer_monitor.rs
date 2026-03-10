use crate::elev_algo::timer::Timer;
use crate::counters::Change;
use crate::world_view::N_NODES;
use serde::{Serialize, Deserialize};
use std::time::Duration;

pub const PEER_TIMEOUT: f64 = Duration::from_millis(500).as_secs_f64(); //If peer misses 5 ticks - time it out.

// Keeps track of and updates availability of all nodes.
#[derive(Clone, Serialize, Deserialize)]
pub struct PeerMonitor {
    pub availability: [bool; N_NODES],
    #[serde(skip)] // Availabilty must be serialized for assigner input and wv transmission. Timers can't and don't need to be serialized
    timers: [Timer; N_NODES], // Holds a timer for each node to test for timeouts
}

impl PeerMonitor {
    pub fn new() -> Self {
        Self {
            availability: [false; N_NODES],
            timers: std::array::from_fn(|_| Timer::new()), // All Timers start as inactive
        }
    }

    // Set availability for a node. Returns true if value flipped
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

    // Called when a message from node with node_id != self_id is received 
    // Resets the node's Timer
    pub fn mark_seen(&mut self, node_id: usize) -> Vec<Change> {
        let mut changes = Vec::new();
        let flipped = self.set(node_id, true);
        
        self.timers[node_id].start(PEER_TIMEOUT); // Sets end_time = now + PEER_TIMEOUT
        if flipped {
            changes.push(Change::PeerAvailability{node_id});
        }
        changes
    }

    // Called at every loop-iteration
    // Check all nodes for timeout. Returns (dead node_ids, changes) 
    pub fn expire_stale_peers(&mut self) -> (Vec<usize>, Vec<Change>) {
        let mut changes = Vec::new();
        let mut dead = Vec::new();

        for node_id in 0..N_NODES {
            if self.is_available(node_id) && self.timers[node_id].timed_out() {
                self.availability[node_id] = false;
                changes.push(Change::PeerAvailability{node_id});
                dead.push(node_id);
            }
        }
        (dead, changes)
    }
}
