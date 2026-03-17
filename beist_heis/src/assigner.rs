use std::collections::HashMap;
use std::process::{Command, Stdio};
use serde::Serialize;
use crate::world_view::*;
use crate::elev_algo::elevator::{N_FLOORS, N_BUTTONS, Dirn, Behaviour, Button};
use crate::orders::{OrderState, OrderTable};


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssignerInput {
    hall_requests: [[bool; N_DIRS]; N_FLOORS],
    states: HashMap<String, ElevatorStateDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ElevatorStateDto {
    behaviour: Behaviour,
    floor: i32,
    direction: Dirn,
    cab_requests: [bool; N_FLOORS],
}

type BinaryOutput = HashMap<String, [[bool; N_BUTTONS]; N_FLOORS]>;

fn build_input(wv: &WorldView) -> AssignerInput {
    let order_table = &wv.order_table;
    let hall_requests = std::array::from_fn(|floor| [
        order_table.hall[floor][0].state == OrderState::Confirmed && order_table.hall[floor][0].assigned_to.is_none(),
        order_table.hall[floor][1].state == OrderState::Confirmed && order_table.hall[floor][1].assigned_to.is_none(),
    ]);
    let self_id = wv.self_id;
    let states = (0..N_NODES)
        .filter(|&id| !wv.node_states.get(id).stuck && (id == self_id || wv.peer_monitor.is_available(id)))
        .map(|id| {
            let elevator_state = wv.node_states.get(id);
            let direction = match (elevator_state.floor, elevator_state.dirn) {
                (0, Dirn::Down) => Dirn::Stop,
                (f, Dirn::Up) if f == N_FLOORS as i32 - 1 => Dirn::Stop,
                _ => elevator_state.dirn,
            };
            let cab_requests = std::array::from_fn(|floor| {
                wv.order_table.get_cab_order(floor, id).state == OrderState::Confirmed
            });
            (id.to_string(), ElevatorStateDto {
                behaviour: elevator_state.behaviour,
                floor: elevator_state.floor,
                direction,
                cab_requests,
            })
        })
        .collect();
    AssignerInput {hall_requests, states}
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
            for btn in [Button::HallUp, Button::HallDown] {
                if node_orders[floor][btn.to_index()] {
                    order_table.hall[floor][btn.to_index()].assigned_to = Some(self_id);
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

        // Confirmed + unassigned — should be included
        wv.order_table.set_hall_order_state(0, Button::HallUp, OrderState::Confirmed);

        // Confirmed + assigned — should be excluded
        wv.order_table.set_hall_order_state(1, Button::HallDown, OrderState::Confirmed);
        wv.order_table.set_hall_order_assigned_to(1, Button::HallDown, Some(1));

        // Unconfirmed — should be excluded
        wv.order_table.set_hall_order_state(2, Button::HallUp, OrderState::Unconfirmed);

        let input = build_input(&wv);

        assert!(input.hall_requests[0][0], "Floor 0 HallUp should be included (confirmed + unassigned)");
        assert!(!input.hall_requests[1][1], "Floor 1 HallDown should be excluded (assigned)");
        assert!(!input.hall_requests[2][0], "Floor 2 HallUp should be excluded (unconfirmed)");
    }

    #[test]
    fn process_output_only_assigns_self_node() {
        let mut order_table = OrderTable::new();

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

        process_assigner_output(1, &binary_output, &mut order_table);

        assert_eq!(order_table.hall[2][1].assigned_to, Some(1), "Floor 2 HallDown should be assigned to node 1 (self)");
        assert_eq!(order_table.hall[0][0].assigned_to, None, "Floor 0 HallUp belongs to node 0, should not be set by node 1's assigner");
    }

    #[test]
    fn process_output_sets_node_id_on_assigned_orders() {
        let mut order_table = OrderTable::new();

        let mut binary_output: BinaryOutput = HashMap::new();
        binary_output.insert("2".to_string(), [
            [false, false, false],
            [false, false, false],
            [false, false, false],
            [true, false, false], // floor 3 HallUp assigned to node 2
        ]);

        process_assigner_output(2, &binary_output, &mut order_table);

        assert_eq!(order_table.hall[3][0].assigned_to, Some(2), "Order should be assigned to node 2");
    }

    #[test]
    fn process_output_discards_cab_orders_from_binary() {
        let mut order_table = OrderTable::new();

        let mut binary_output: BinaryOutput = HashMap::new();
        // Binary includes cab order (index 2) - should be ignored
        binary_output.insert("0".to_string(), [
            [false, false, true], // cab order at floor 0
            [false, false, false],
            [false, false, false],
            [false, false, false],
        ]);

        process_assigner_output(0, &binary_output, &mut order_table);

        for floor in 0..N_FLOORS {
            assert_eq!(order_table.hall[floor][0].assigned_to, None, "No HallUp should be assigned");
            assert_eq!(order_table.hall[floor][1].assigned_to, None, "No HallDown should be assigned");
        }
    }
}
