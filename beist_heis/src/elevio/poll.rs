use crossbeam_channel as cbc;
use std::thread;
use std::time;

use super::elev;
use crate::elev_algo::fsm::SensorEvent;
use crate::elev_algo::elevator::{N_BUTTONS, N_FLOORS};

// Based on TTK4145 course github
// Authors: Henrik Horluck, klasbo
// Availability: https://github.com/TTK4145/driver-rust

#[derive(Debug, Clone)]
pub struct ButtonEvent {
    pub floor: u8,
    pub button: u8,
}

pub fn poll_buttons(elev: elev::Elevator, ch: cbc::Sender<ButtonEvent>, period: time::Duration) {
    let mut prev = [[false; N_BUTTONS]; N_FLOORS];
    loop {
        for floor in 0..N_FLOORS as u8{
            for btn in 0..N_BUTTONS as u8{
                let pressed = elev.call_button(floor, btn);
                if pressed && prev[floor as usize][btn as usize] != pressed {
                    ch.send(ButtonEvent {floor: floor, button: btn}).unwrap();
                }
                prev[floor as usize][btn as usize] = pressed;
            }
        }
        thread::sleep(period)
    }
}

pub fn poll_sensors(elev: elev::Elevator, ch: cbc::Sender<SensorEvent>, period: time::Duration) {
    let mut prev_floor = u8::MAX;
    let mut prev_obstr = false;
    let mut prev_stop = false;

    loop {
        if let Some(floor) = elev.floor_sensor() {
            if floor != prev_floor {
                ch.send(SensorEvent::FloorArrival(floor)).unwrap();
                prev_floor = floor;
            }
        }
        let obstr = elev.obstruction();
        if obstr != prev_obstr {
            ch.send(SensorEvent::Obstruction(obstr)).unwrap();
            prev_obstr = obstr;
        }
        // Stop button unused
        let stop = elev.stop_button();
        if stop != prev_stop {
            ch.send(SensorEvent::StopButton(stop)).unwrap();
            prev_stop = stop;
        }
        thread::sleep(period)
    }
}
