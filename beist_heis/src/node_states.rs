use crate::elev_algo::elevator::Elevator;
use crate::world_view::N_NODES;
use serde::{Serialize, Deserialize};
use crate::counters::Change;

// Holds internal states for all nodes
#[derive(Clone, Serialize, Deserialize)]
pub struct NodeStates {
    pub node_states: [Elevator; N_NODES],
}

impl NodeStates {
    pub fn new() -> Self {
        Self {
            node_states: [Elevator::new(); N_NODES] 
        }
    }

    pub fn get(&self, node_id: usize) -> Elevator {
        self.node_states[node_id]
    }
    
    pub fn set(&mut self, node_id: usize, elevator: Elevator) -> Vec<Change> {
        self.node_states[node_id] = elevator;
        vec![Change::Elevator { node_id }]
    }
}
