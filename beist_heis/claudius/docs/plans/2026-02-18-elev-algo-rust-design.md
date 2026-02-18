# Elevator Algorithm Rust Translation Design

## Overview

Translate the C elevator algorithm from `../Project-resources/elev_algo/` to idiomatic Rust as a standalone library module. Hardware integration will be handled separately by the user.

## Scope

**Included:**
- Data structures (Elevator, Dirn, Button, Behaviour)
- Pure request algorithms
- FSM event handlers
- Timer

**Excluded:**
- Hardware I/O (handled externally)
- Main polling loop (user integrates)

## Module Structure

```
elev_algo/
├── Cargo.toml
└── src/
    ├── lib.rs        # Re-exports public API
    ├── elevator.rs   # Elevator, Dirn, Button, Behaviour, constants
    ├── requests.rs   # Pure request algorithms (impl on Elevator)
    ├── fsm.rs        # FSM event handlers, FsmOutput
    └── timer.rs      # Timer
```

## Core Types (elevator.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dirn {
    Down = -1,
    Stop = 0,
    Up = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    HallUp,
    HallDown,
    Cab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Behaviour {
    #[default]
    Idle,
    Moving,
    DoorOpen,
}

pub const N_FLOORS: usize = 4;
pub const N_BUTTONS: usize = 3;

#[derive(Debug, Clone)]
pub struct Elevator {
    pub floor: i32,                              // -1 if between floors
    pub dirn: Dirn,
    pub requests: [[bool; N_BUTTONS]; N_FLOORS], // true/false instead of int
    pub behaviour: Behaviour,
    pub door_open_duration_s: f64,
}
```

## Request Algorithms (requests.rs)

```rust
#[derive(Debug, Clone, Copy)]
pub struct DirnBehaviour {
    pub dirn: Dirn,
    pub behaviour: Behaviour,
}

impl Elevator {
    pub fn requests_above(&self) -> bool;
    pub fn requests_below(&self) -> bool;
    pub fn requests_here(&self) -> bool;
    pub fn choose_direction(&self) -> DirnBehaviour;
    pub fn should_stop(&self) -> bool;
    pub fn should_clear_immediately(&self, floor: usize, button: Button) -> bool;
    pub fn clear_at_current_floor(&self) -> Self;
}
```

All functions are pure - they return new state rather than mutating.

## FSM Event Handlers (fsm.rs)

```rust
#[derive(Debug, Clone)]
pub struct FsmOutput {
    pub motor_direction: Option<Dirn>,
    pub door_light: Option<bool>,
    pub start_door_timer: bool,
    pub clear_lights: Vec<(usize, Button)>,
}

impl Elevator {
    pub fn on_request_button_press(&self, floor: usize, button: Button) -> (Self, FsmOutput);
    pub fn on_floor_arrival(&self, floor: usize) -> (Self, FsmOutput);
    pub fn on_door_timeout(&self) -> (Self, FsmOutput);
}
```

Pure functions returning (new_state, output_actions). User interprets FsmOutput for hardware calls.

## Timer (timer.rs)

```rust
pub struct Timer {
    start_time: Option<Instant>,
    duration_secs: f64,
}

impl Timer {
    pub fn new() -> Self;
    pub fn start(&mut self, duration_secs: f64);
    pub fn timed_out(&self) -> bool;
    pub fn stop(&mut self);
}
```

Uses `std::time::Instant`.

## Usage Example

```rust
use elev_algo::{Elevator, Timer, Button};

let mut elevator = Elevator::new();
let mut timer = Timer::new();

// On button press
let (elevator, output) = elevator.on_request_button_press(2, Button::Cab);
if output.start_door_timer {
    timer.start(elevator.door_open_duration_s);
}
// Handle output.motor_direction, output.door_light, etc.
```
