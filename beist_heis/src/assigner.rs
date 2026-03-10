use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::fs;
use serde::Serialize;
use crate::world_view::*;
use crate::elev_algo::elevator::{N_FLOORS, Dirn, Behaviour, Button};
use crate::orders::{OrderState, OrderTable, UNASSIGNED_NODE};


#[derive(Serialize)]
struct AssignerInput {
    #[serde(rename = "hallRequests")]
    hall_requests: [[bool; 2]; N_FLOORS],
    states: HashMap<String, ElevatorStateDto>,
}

#[derive(Serialize)]
struct ElevatorStateDto {
    behaviour: Behaviour,
    floor: i32,
    direction: Dirn,
    #[serde(rename = "cabRequests")]
    cab_requests: [bool; N_FLOORS],
}

type BinaryOutput = HashMap<String, [[bool; 3]; N_FLOORS]>;

fn build_input(wv: &WorldView) -> AssignerInput {
    let ot = &wv.order_table;
    // Only include orders where state == Confirmed AND node_id == UNASSIGNED_NODE
    let hall_requests = std::array::from_fn(|floor| [
        ot.hall[floor][0].state == OrderState::Confirmed && ot.hall[floor][0].node_id == UNASSIGNED_NODE,
        ot.hall[floor][1].state == OrderState::Confirmed && ot.hall[floor][1].node_id == UNASSIGNED_NODE,
    ]);
    let self_id = wv.self_id;
    let states = (0..N_NODES)
        .filter(|&id| !wv.elevator_map.get(id).stuck && (id == self_id || wv.peer_monitor.is_available(id)))
        .map(|id| {
            let e = wv.elevator_map.get(id);
            let direction = match (e.floor, e.dirn) {
                (0, Dirn::Down)                                    => Dirn::Stop,
                (f, Dirn::Up) if f == N_FLOORS as i32 - 1         => Dirn::Stop,
                _                                                  => e.dirn,
            };
            // Include already-assigned hall orders as if they were cab requests so the
            // cost function sees the full load each elevator is carrying.  Without this
            // the binary treats an elevator that already owns hall orders as if it were
            // empty, causing all redistributed orders to pile onto the same elevator.
            let cab_requests = std::array::from_fn(|floor| {
                e.requests[floor][2]
                    || ot.hall[floor][0].node_id == id
                    || ot.hall[floor][1].node_id == id
            });
            (id.to_string(), ElevatorStateDto {
                behaviour: e.behaviour,
                floor: e.floor,
                direction,
                cab_requests,
            })
        })
        .collect();
    AssignerInput { hall_requests, states }
}

/// Run the hall request assigner binary and return a copy of the order table
/// with node_ids set for orders assigned to this node.
/// Takes an immutable snapshot — call from a dedicated thread.
pub fn assign_hall_requests(
    wv: &WorldView,
    assigner_path: &str,
) -> Result<OrderTable, Box<dyn std::error::Error>> {
    let input_json = serde_json::to_string(&build_input(wv))?;

    let output = Command::new(assigner_path)
        .arg("--input")
        .arg(&input_json)
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "hall_request_assigner failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    let binary_output: BinaryOutput = serde_json::from_slice(&output.stdout)?;
    let mut order_table = wv.order_table.clone();
    process_assigner_output(wv.self_id, &binary_output, &mut order_table);
    Ok(order_table)
}

pub fn save_assigner_input(wv: &WorldView, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input = build_input(wv);
    fs::write(path, serde_json::to_string_pretty(&input)?)?;
    Ok(())
}

/// Apply the assigner's binary output to `order_table` for `self_id` only.
/// Each node runs this independently; other nodes' assignments propagate via WV merging.
/// Extracted for testability without requiring the external binary.
pub fn process_assigner_output(
    self_id: usize,
    binary_output: &BinaryOutput,
    order_table: &mut OrderTable,
) {
    if let Some(node_orders) = binary_output.get(&self_id.to_string()) {
        for floor in 0..N_FLOORS {
            for btn in 0..2 {
                if node_orders[floor][btn] {
                    order_table.hall[floor][btn].node_id = self_id;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elev_algo::elevator::Button;

    #[test]
    fn build_input_only_includes_unassigned_confirmed_orders() {
        let mut wv = WorldView::new(0);

        // Set up: floor 0 HallUp = Confirmed + UNASSIGNED (should include)
        wv.order_table.set_hall_order_state(0, Button::HallUp, OrderState::Confirmed);
        // node_id defaults to UNASSIGNED_NODE

        // Set up: floor 1 HallDown = Confirmed but assigned to node 1 (should exclude)
        wv.order_table.set_hall_order_state(1, Button::HallDown, OrderState::Confirmed);
        wv.order_table.set_hall_order_node_id(1, Button::HallDown, 1);

        // Set up: floor 2 HallUp = Unconfirmed + UNASSIGNED (should exclude)
        wv.order_table.set_hall_order_state(2, Button::HallUp, OrderState::Unconfirmed);

        let input = build_input(&wv);

        // Floor 0 HallUp should be true (confirmed + unassigned)
        assert!(input.hall_requests[0][0], "Floor 0 HallUp should be included");

        // Floor 1 HallDown should be false (confirmed but assigned)
        assert!(!input.hall_requests[1][1], "Floor 1 HallDown should be excluded (assigned)");

        // Floor 2 HallUp should be false (unconfirmed)
        assert!(!input.hall_requests[2][0], "Floor 2 HallUp should be excluded (unconfirmed)");
    }

    #[test]
    fn process_output_only_assigns_self_node() {
        let mut ot = OrderTable::new();

        let mut binary_output: BinaryOutput = HashMap::new();
        // Node 0 gets floor 0 HallUp
        binary_output.insert("0".to_string(), [
            [true, false, false],
            [false, false, false],
            [false, false, false],
            [false, false, false],
        ]);
        // Node 1 (self) gets floor 2 HallDown
        binary_output.insert("1".to_string(), [
            [false, false, false],
            [false, false, false],
            [false, true, false],
            [false, false, false],
        ]);

        process_assigner_output(1, &binary_output, &mut ot);

        assert_eq!(ot.hall[2][1].node_id, 1, "Floor 2 HallDown should be assigned to node 1 (self)");
        assert_eq!(ot.hall[0][0].node_id, UNASSIGNED_NODE, "Floor 0 HallUp belongs to node 0, should not be set by node 1's assigner");
    }

    #[test]
    fn process_output_sets_node_id_on_assigned_orders() {
        let mut ot = OrderTable::new();

        let mut binary_output: BinaryOutput = HashMap::new();
        binary_output.insert("2".to_string(), [
            [false, false, false],
            [false, false, false],
            [false, false, false],
            [true, false, false], // floor 3 HallUp assigned to node 2
        ]);

        process_assigner_output(2, &binary_output, &mut ot);

        assert_eq!(ot.hall[3][0].node_id, 2, "Order should be assigned to node 2");
    }

    #[test]
    fn process_output_discards_cab_orders_from_binary() {
        let mut ot = OrderTable::new();

        let mut binary_output: BinaryOutput = HashMap::new();
        // Binary includes cab order (index 2) - should be ignored
        binary_output.insert("0".to_string(), [
            [false, false, true], // cab order at floor 0
            [false, false, false],
            [false, false, false],
            [false, false, false],
        ]);

        process_assigner_output(0, &binary_output, &mut ot);

        for floor in 0..N_FLOORS {
            assert_eq!(ot.hall[floor][0].node_id, UNASSIGNED_NODE, "No HallUp should be assigned");
            assert_eq!(ot.hall[floor][1].node_id, UNASSIGNED_NODE, "No HallDown should be assigned");
        }
    }
}
