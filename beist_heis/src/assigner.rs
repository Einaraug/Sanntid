use std::collections::HashMap;
use std::process::{Command, Stdio};
use serde::Serialize;
use crate::world_view::*;
use crate::elev_algo::elevator::{N_FLOORS, N_BUTTONS, Dirn, Behaviour, Button};
use crate::orders::{OrderState, OrderTable};
use crossbeam_channel as cbc;

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

pub fn assign_hall_requests(wv: &WorldView, assigner_path: &str) -> Result<OrderTable, Box<dyn std::error::Error>> {
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

pub fn process_assigner_output(self_id: usize, binary_output: &BinaryOutput, order_table: &mut OrderTable) {
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

pub fn run(rx: cbc::Receiver<WorldView>, tx: cbc::Sender<OrderTable>, path: &str) {
    for wv_snapshot in rx {
        match assign_hall_requests(&wv_snapshot, path) {
            Ok(order_table) => { let _ = tx.send(order_table); }
            Err(e) => eprintln!("assigner: {e}"),
        }
    }
}