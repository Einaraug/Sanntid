use beist_heis::elevio::elev::*;
use beist_heis::elevio::poll::*;

use std::time::Duration;
use std::thread;

fn main() {
    let elev = Elevator::init("localhost:15657", 4).expect("Failed to connect to elevator server");
    println!("Connected: {}", elev);

    // Go up
    println!("Moving up...");
    elev.motor_direction(DIRN_UP);
    thread::sleep(Duration::from_secs(3));

    // Stop
    println!("Stopping...");
    elev.motor_direction(DIRN_STOP);
    thread::sleep(Duration::from_secs(1));

    // Go down
    println!("Moving down...");
    elev.motor_direction(DIRN_DOWN);
    thread::sleep(Duration::from_secs(3));

    // Stop and test lights
    elev.motor_direction(DIRN_STOP);
    println!("Testing lights...");
    elev.floor_indicator(0);
    elev.door_light(true);
    thread::sleep(Duration::from_secs(1));
    elev.door_light(false);

    // Read sensors
    println!("Floor sensor: {:?}", elev.floor_sensor());
    println!("Stop button: {}", elev.stop_button());
    println!("Obstruction: {}", elev.obstruction());
}