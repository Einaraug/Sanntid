// ── ElevatorMap ─

#[derive(Clone, Serialize, Deserialize)]
pub struct Nodes {
    pub nodes: [Elevator; N_NODES],
}

impl Nodes {
    pub fn new() -> Self {
        Self {nodes: [Elevator::new(); N_NODES] }
    }
    pub fn get(&self, node_id: usize) -> Elevator {
        self.elevator[node_id]
    }
    pub fn set(&mut self, node_id: usize, elevator: Elevator) -> Vec<Change> {
        self.elevator[node_id] = elevator;
        vec![Change::Elevator { node_id }]
    }
}
