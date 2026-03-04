use crossbeam_channel as cbc;
use std::thread;
use std::time;

use super::elev;
use crate::elev_algo::fsm::SensorEvent;

/// Button press event - sent to WorldView
#[derive(Debug, Clone)]
pub struct ButtonEvent {
    pub floor: u8,
    pub button: u8,
}

/// Polls buttons and sends to WorldView
pub fn poll_buttons(elev: elev::Elevator, ch: cbc::Sender<ButtonEvent>, period: time::Duration) {
    let mut prev = vec![[false; 3]; elev.num_floors.into()];
    loop {
        for f in 0..elev.num_floors {
            for c in 0..3 {
                let v = elev.call_button(f, c);
                if v && prev[f as usize][c as usize] != v {
                    ch.send(ButtonEvent { floor: f, button: c }).unwrap();
                }
                prev[f as usize][c as usize] = v;
            }
        }
        thread::sleep(period)
    }
}

/// Polls floor sensor, obstruction, stop button - sends directly to FSM
pub fn poll_sensors(elev: elev::Elevator, ch: cbc::Sender<SensorEvent>, period: time::Duration) {
    let mut prev_floor = u8::MAX;
    let mut prev_obstr = false;
    let mut prev_stop = false;

    loop {
        // Floor sensor
        if let Some(f) = elev.floor_sensor() {
            if f != prev_floor {
                ch.send(SensorEvent::FloorArrival(f)).unwrap();
                prev_floor = f;
            }
        }

        // Obstruction
        let obstr = elev.obstruction();
        if obstr != prev_obstr {
            ch.send(SensorEvent::Obstruction(obstr)).unwrap();
            prev_obstr = obstr;
        }

        // Stop button
        let stop = elev.stop_button();
        if stop != prev_stop {
            ch.send(SensorEvent::StopButton(stop)).unwrap();
            prev_stop = stop;
        }

        thread::sleep(period)
    }
}
