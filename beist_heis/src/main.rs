// main.rs
mod elevio;
mod elev_algo;

use elevio::elev as hw;
use elevio::poll;
use elev_algo::elevator::Elevator;
use elev_algo::timer::Timer;
use crossbeam_channel as cbc;
use std::thread;
use std::time::Duration;

fn main() {
    let hw_elev = hw::Elevator::init("localhost:15657", 4).unwrap();
    let mut elev = Elevator::new();
    let mut timer = Timer::new();

    // Spawn poll threads
    let (floor_tx, floor_rx) = cbc::unbounded();
    let (btn_tx, btn_rx) = cbc::unbounded();

    let hw2 = hw_elev.clone();
    thread::spawn(move || poll::floor_sensor(hw2, floor_tx, Duration::from_millis(25)));
    let hw3 = hw_elev.clone();
    thread::spawn(move || poll::call_buttons(hw3, btn_tx, Duration::from_millis(25)));

    // Init: go down if between floors
    if hw_elev.floor_sensor().is_none() {
        let (new_elev, output) = elev.on_init_between_floors();
        elev = new_elev;
        apply_output(&hw_elev, &output);
    }

    // Main event loop
    loop {
        cbc::select! {
            recv(floor_rx) -> floor => {
                let floor = floor.unwrap();
                let (new_elev, output) = elev.on_floor_arrival(floor as i32);
                elev = new_elev;
                apply_output(&hw_elev, &output);
                if output.start_door_timer {
                    timer.start(elev.door_open_duration_s);
                }
            }
            recv(btn_rx) -> btn => {
                let btn = btn.unwrap();
                let button = match btn.call {
                    0 => elev_algo::elevator::Button::HallUp,
                    1 => elev_algo::elevator::Button::HallDown,
                    _ => elev_algo::elevator::Button::Cab,
                };
                let (new_elev, output) = elev.on_request_button_press(btn.floor as usize, button);
                elev = new_elev;
                apply_output(&hw_elev, &output);
            }
            default(Duration::from_millis(25)) => {
                if timer.timed_out() {
                    timer.stop();
                    let (new_elev, output) = elev.on_door_timeout();
                    elev = new_elev;
                    apply_output(&hw_elev, &output);
                }
            }
        }
    }
}

fn apply_output(hw: &hw::Elevator, output: &elev_algo::fsm::FsmOutput) {
    if let Some(dirn) = output.motor_direction {
        hw.motor_direction(match dirn {
            elev_algo::elevator::Dirn::Up   => hw::DIRN_UP,
            elev_algo::elevator::Dirn::Down => hw::DIRN_DOWN,
            elev_algo::elevator::Dirn::Stop => hw::DIRN_STOP,
        });
    }
    if let Some(on) = output.door_light {
        hw.door_light(on);
    }
    if let Some(floor) = output.floor_indicator {
        hw.floor_indicator(floor as u8);
    }
    for (floor, btn) in &output.clear_lights {
        hw.call_button_light(*floor as u8, btn.to_index() as u8, false);
    }
    for (floor, btn) in &output.set_lights {
        hw.call_button_light(*floor as u8, btn.to_index() as u8, true);
    }
}