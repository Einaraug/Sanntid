use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::fs;
use serde::Serialize;
use crate::world_view::*;
use crate::elev_algo::elevator::{N_FLOORS, Dirn, Behaviour};
use crate::orders::OrderState;


#[derive(Serialize)]
struct AssignerInput { // Replace with world_view::ElevatorMap
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


pub type AssignerOutput = HashMap<String, [[bool; 3]; N_FLOORS]>;

fn build_input(wv: &WorldView) -> AssignerInput {
    let ot = wv.get_order_table();
    let hall_requests = std::array::from_fn(|floor| [
        ot.hall[floor][0].state == OrderState::Confirmed,
        ot.hall[floor][1].state == OrderState::Confirmed,
    ]);
    let states = (0..(N_NODES))
        .filter(|&id| wv.get_peer_availability().is_available(id))
        .map(|id| {
            let e = wv.get_elevator_map().get_elevator(id);
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
    wv: &WorldView,
    assigner_path: &str,
) -> Result<AssignerOutput, Box<dyn std::error::Error>> {
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

    Ok(serde_json::from_slice(&output.stdout)?)
}



pub fn save_assigner_input(wv: &WorldView, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input = build_input(wv);
    fs::write(path, serde_json::to_string_pretty(&input)?)?;
    Ok(())
}


