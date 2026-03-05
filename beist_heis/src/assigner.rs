use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::fs;
use serde::Serialize;
use crate::world_view::*;
use crate::elev_algo::elevator::{N_FLOORS, Dirn, Behaviour, Button};
use crate::orders::{OrderState, UNASSIGNED_NODE};


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
    let ot = wv.get_order_table();
    // Only include orders where state == Confirmed AND node_id == UNASSIGNED_NODE
    let hall_requests = std::array::from_fn(|floor| [
        ot.hall[floor][0].state == OrderState::Confirmed && ot.hall[floor][0].node_id == UNASSIGNED_NODE,
        ot.hall[floor][1].state == OrderState::Confirmed && ot.hall[floor][1].node_id == UNASSIGNED_NODE,
    ]);
    let states = (0..N_NODES)
        .filter(|&id| wv.get_peer_availability().get(id))
        .map(|id| {
            let e = wv.get_elevator_map().get(id);
            (id.to_string(), ElevatorStateDto {
                behaviour: e.behaviour,
                floor: e.floor,
                direction: e.dirn,
                cab_requests: e.requests.map(|floor_btns| floor_btns[2]),
            })
        })
        .collect();
    AssignerInput { hall_requests, states }
}

pub fn assign_hall_requests(
    wv: &mut WorldView,
    assigner_path: &str,
) -> Result<[[bool; 2]; N_FLOORS], Box<dyn std::error::Error>> {
    let input_json = serde_json::to_string(&build_input(wv))?;

    let mut child = Command::new(assigner_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child.stdin.as_mut().unwrap().write_all(input_json.as_bytes())?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(format!(
            "hall_request_assigner failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    let binary_output: BinaryOutput = serde_json::from_slice(&output.stdout)?;
    Ok(process_assigner_output(wv, &binary_output))
}

pub fn save_assigner_input(wv: &WorldView, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input = build_input(wv);
    fs::write(path, serde_json::to_string_pretty(&input)?)?;
    Ok(())
}

/// Process binary output and update WorldView, returning orders for this node.
/// Extracted for testability without requiring the external binary.
pub fn process_assigner_output(
    wv: &mut WorldView,
    binary_output: &BinaryOutput,
) -> [[bool; 2]; N_FLOORS] {
    let self_id = *wv.get_self_id();
    let self_id_str = self_id.to_string();

    let mut result = [[false; 2]; N_FLOORS];

    if let Some(node_orders) = binary_output.get(&self_id_str) {
        for floor in 0..N_FLOORS {
            for btn in 0..2 {
                if node_orders[floor][btn] {
                    result[floor][btn] = true;
                    let button = Button::from_index(btn).unwrap();
                    wv.set_hall_order_node_id(floor, button, self_id);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elev_algo::elevator::Button;

    #[test]
    fn build_input_only_includes_unassigned_confirmed_orders() {
        let mut wv = WorldView::new(0);

        // Set up: floor 0 HallUp = Confirmed + UNASSIGNED (should include)
        wv.set_hall_order_state(0, Button::HallUp, OrderState::Confirmed);
        // node_id defaults to UNASSIGNED_NODE

        // Set up: floor 1 HallDown = Confirmed but assigned to node 1 (should exclude)
        wv.set_hall_order_state(1, Button::HallDown, OrderState::Confirmed);
        wv.set_hall_order_node_id(1, Button::HallDown, 1);

        // Set up: floor 2 HallUp = Unconfirmed + UNASSIGNED (should exclude)
        wv.set_hall_order_state(2, Button::HallUp, OrderState::Unconfirmed);

        let input = build_input(&wv);

        // Floor 0 HallUp should be true (confirmed + unassigned)
        assert!(input.hall_requests[0][0], "Floor 0 HallUp should be included");

        // Floor 1 HallDown should be false (confirmed but assigned)
        assert!(!input.hall_requests[1][1], "Floor 1 HallDown should be excluded (assigned)");

        // Floor 2 HallUp should be false (unconfirmed)
        assert!(!input.hall_requests[2][0], "Floor 2 HallUp should be excluded (unconfirmed)");
    }

    #[test]
    fn process_output_extracts_orders_for_self_node_only() {
        let mut wv = WorldView::new(1); // self_id = 1

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

        let result = process_assigner_output(&mut wv, &binary_output);

        // Should only have floor 2 HallDown for node 1
        assert!(!result[0][0], "Floor 0 HallUp belongs to node 0, not us");
        assert!(result[2][1], "Floor 2 HallDown should be assigned to us (node 1)");
    }

    #[test]
    fn process_output_sets_node_id_on_assigned_orders() {
        let mut wv = WorldView::new(2); // self_id = 2

        // Set up confirmed unassigned order at floor 3 HallUp
        wv.set_hall_order_state(3, Button::HallUp, OrderState::Confirmed);

        // Verify it starts as unassigned
        assert_eq!(
            wv.get_order_table().hall[3][0].node_id,
            UNASSIGNED_NODE,
            "Order should start unassigned"
        );

        let mut binary_output: BinaryOutput = HashMap::new();
        binary_output.insert("2".to_string(), [
            [false, false, false],
            [false, false, false],
            [false, false, false],
            [true, false, false], // floor 3 HallUp assigned to node 2
        ]);

        let _result = process_assigner_output(&mut wv, &binary_output);

        // Verify node_id was set
        assert_eq!(
            wv.get_order_table().hall[3][0].node_id,
            2,
            "Order should now be assigned to node 2"
        );
    }

    #[test]
    fn process_output_discards_cab_orders_from_binary() {
        let mut wv = WorldView::new(0);

        let mut binary_output: BinaryOutput = HashMap::new();
        // Binary includes cab order (index 2) - should be ignored
        binary_output.insert("0".to_string(), [
            [false, false, true], // cab order at floor 0
            [false, false, false],
            [false, false, false],
            [false, false, false],
        ]);

        let result = process_assigner_output(&mut wv, &binary_output);

        // Result should be empty (cab orders discarded)
        for floor in 0..N_FLOORS {
            assert!(!result[floor][0], "No HallUp orders");
            assert!(!result[floor][1], "No HallDown orders");
        }
    }
}
