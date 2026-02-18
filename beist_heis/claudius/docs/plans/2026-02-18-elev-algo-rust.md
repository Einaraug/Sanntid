# Elevator Algorithm Rust Translation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Translate the C elevator algorithm to idiomatic Rust as a standalone library.

**Architecture:** Pure functional approach where FSM handlers return (new_state, outputs) instead of mutating state and calling hardware directly. All request algorithms are methods on `Elevator`.

**Tech Stack:** Rust stable, std::time::Instant for timer, no external dependencies.

---

### Task 1: Project Setup

**Files:**
- Create: `elev_algo/Cargo.toml`
- Create: `elev_algo/src/lib.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "elev_algo"
version = "0.1.0"
edition = "2021"

[dependencies]
```

**Step 2: Create lib.rs with module declarations**

```rust
mod elevator;
mod requests;
mod fsm;
mod timer;

pub use elevator::*;
pub use requests::*;
pub use fsm::*;
pub use timer::*;
```

**Step 3: Verify project compiles**

Run: `cd elev_algo && cargo check`
Expected: Error about missing modules (expected at this stage)

**Step 4: Commit**

```bash
git add elev_algo/
git commit -m "chore: scaffold elev_algo Rust crate"
```

---

### Task 2: Core Types (elevator.rs)

**Files:**
- Create: `elev_algo/src/elevator.rs`

**Step 1: Write test for enum conversions**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirn_values() {
        assert_eq!(Dirn::Down as i32, -1);
        assert_eq!(Dirn::Stop as i32, 0);
        assert_eq!(Dirn::Up as i32, 1);
    }

    #[test]
    fn test_button_to_index() {
        assert_eq!(Button::HallUp.to_index(), 0);
        assert_eq!(Button::HallDown.to_index(), 1);
        assert_eq!(Button::Cab.to_index(), 2);
    }

    #[test]
    fn test_elevator_new() {
        let e = Elevator::new();
        assert_eq!(e.floor, -1);
        assert_eq!(e.dirn, Dirn::Stop);
        assert_eq!(e.behaviour, Behaviour::Idle);
        assert_eq!(e.door_open_duration_s, 3.0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL - types not defined

**Step 3: Implement types**

```rust
pub const N_FLOORS: usize = 4;
pub const N_BUTTONS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
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

impl Button {
    pub fn to_index(self) -> usize {
        match self {
            Button::HallUp => 0,
            Button::HallDown => 1,
            Button::Cab => 2,
        }
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Button::HallUp),
            1 => Some(Button::HallDown),
            2 => Some(Button::Cab),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Behaviour {
    #[default]
    Idle,
    DoorOpen,
    Moving,
}

#[derive(Debug, Clone)]
pub struct Elevator {
    pub floor: i32,
    pub dirn: Dirn,
    pub requests: [[bool; N_BUTTONS]; N_FLOORS],
    pub behaviour: Behaviour,
    pub door_open_duration_s: f64,
}

impl Elevator {
    pub fn new() -> Self {
        Self {
            floor: -1,
            dirn: Dirn::Stop,
            requests: [[false; N_BUTTONS]; N_FLOORS],
            behaviour: Behaviour::Idle,
            door_open_duration_s: 3.0,
        }
    }
}

impl Default for Elevator {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add core types Dirn, Button, Behaviour, Elevator"
```

---

### Task 3: Request Helper Functions

**Files:**
- Create: `elev_algo/src/requests.rs`

**Step 1: Write tests for requests_above/below/here**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Elevator, Button, Dirn, Behaviour};

    fn elevator_at_floor(floor: i32) -> Elevator {
        let mut e = Elevator::new();
        e.floor = floor;
        e
    }

    #[test]
    fn test_requests_above_empty() {
        let e = elevator_at_floor(0);
        assert!(!e.requests_above());
    }

    #[test]
    fn test_requests_above_with_request() {
        let mut e = elevator_at_floor(0);
        e.requests[2][Button::Cab.to_index()] = true;
        assert!(e.requests_above());
    }

    #[test]
    fn test_requests_above_only_below() {
        let mut e = elevator_at_floor(2);
        e.requests[0][Button::Cab.to_index()] = true;
        assert!(!e.requests_above());
    }

    #[test]
    fn test_requests_below_empty() {
        let e = elevator_at_floor(3);
        assert!(!e.requests_below());
    }

    #[test]
    fn test_requests_below_with_request() {
        let mut e = elevator_at_floor(3);
        e.requests[1][Button::HallUp.to_index()] = true;
        assert!(e.requests_below());
    }

    #[test]
    fn test_requests_here_empty() {
        let e = elevator_at_floor(1);
        assert!(!e.requests_here());
    }

    #[test]
    fn test_requests_here_with_request() {
        let mut e = elevator_at_floor(1);
        e.requests[1][Button::Cab.to_index()] = true;
        assert!(e.requests_here());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL - methods not defined

**Step 3: Implement helper methods**

```rust
use crate::elevator::{Elevator, N_FLOORS, N_BUTTONS};

impl Elevator {
    pub fn requests_above(&self) -> bool {
        for f in (self.floor + 1) as usize..N_FLOORS {
            for btn in 0..N_BUTTONS {
                if self.requests[f][btn] {
                    return true;
                }
            }
        }
        false
    }

    pub fn requests_below(&self) -> bool {
        if self.floor <= 0 {
            return false;
        }
        for f in 0..self.floor as usize {
            for btn in 0..N_BUTTONS {
                if self.requests[f][btn] {
                    return true;
                }
            }
        }
        false
    }

    pub fn requests_here(&self) -> bool {
        if self.floor < 0 {
            return false;
        }
        for btn in 0..N_BUTTONS {
            if self.requests[self.floor as usize][btn] {
                return true;
            }
        }
        false
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add requests_above/below/here helpers"
```

---

### Task 4: choose_direction Algorithm

**Files:**
- Modify: `elev_algo/src/requests.rs`

**Step 1: Write tests for choose_direction**

```rust
use crate::{Dirn, Behaviour};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirnBehaviour {
    pub dirn: Dirn,
    pub behaviour: Behaviour,
}

// Add to tests module:
#[test]
fn test_choose_direction_idle_no_requests() {
    let e = elevator_at_floor(1);
    let result = e.choose_direction();
    assert_eq!(result.dirn, Dirn::Stop);
    assert_eq!(result.behaviour, Behaviour::Idle);
}

#[test]
fn test_choose_direction_going_up_requests_above() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Up;
    e.requests[3][Button::Cab.to_index()] = true;
    let result = e.choose_direction();
    assert_eq!(result.dirn, Dirn::Up);
    assert_eq!(result.behaviour, Behaviour::Moving);
}

#[test]
fn test_choose_direction_going_up_requests_here() {
    let mut e = elevator_at_floor(2);
    e.dirn = Dirn::Up;
    e.requests[2][Button::HallDown.to_index()] = true;
    let result = e.choose_direction();
    assert_eq!(result.dirn, Dirn::Down);
    assert_eq!(result.behaviour, Behaviour::DoorOpen);
}

#[test]
fn test_choose_direction_going_up_requests_below() {
    let mut e = elevator_at_floor(3);
    e.dirn = Dirn::Up;
    e.requests[0][Button::Cab.to_index()] = true;
    let result = e.choose_direction();
    assert_eq!(result.dirn, Dirn::Down);
    assert_eq!(result.behaviour, Behaviour::Moving);
}

#[test]
fn test_choose_direction_stopped_request_here() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Stop;
    e.requests[1][Button::Cab.to_index()] = true;
    let result = e.choose_direction();
    assert_eq!(result.dirn, Dirn::Stop);
    assert_eq!(result.behaviour, Behaviour::DoorOpen);
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL - choose_direction not defined

**Step 3: Implement choose_direction**

```rust
impl Elevator {
    pub fn choose_direction(&self) -> DirnBehaviour {
        match self.dirn {
            Dirn::Up => {
                if self.requests_above() {
                    DirnBehaviour { dirn: Dirn::Up, behaviour: Behaviour::Moving }
                } else if self.requests_here() {
                    DirnBehaviour { dirn: Dirn::Down, behaviour: Behaviour::DoorOpen }
                } else if self.requests_below() {
                    DirnBehaviour { dirn: Dirn::Down, behaviour: Behaviour::Moving }
                } else {
                    DirnBehaviour { dirn: Dirn::Stop, behaviour: Behaviour::Idle }
                }
            }
            Dirn::Down => {
                if self.requests_below() {
                    DirnBehaviour { dirn: Dirn::Down, behaviour: Behaviour::Moving }
                } else if self.requests_here() {
                    DirnBehaviour { dirn: Dirn::Up, behaviour: Behaviour::DoorOpen }
                } else if self.requests_above() {
                    DirnBehaviour { dirn: Dirn::Up, behaviour: Behaviour::Moving }
                } else {
                    DirnBehaviour { dirn: Dirn::Stop, behaviour: Behaviour::Idle }
                }
            }
            Dirn::Stop => {
                if self.requests_here() {
                    DirnBehaviour { dirn: Dirn::Stop, behaviour: Behaviour::DoorOpen }
                } else if self.requests_above() {
                    DirnBehaviour { dirn: Dirn::Up, behaviour: Behaviour::Moving }
                } else if self.requests_below() {
                    DirnBehaviour { dirn: Dirn::Down, behaviour: Behaviour::Moving }
                } else {
                    DirnBehaviour { dirn: Dirn::Stop, behaviour: Behaviour::Idle }
                }
            }
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add choose_direction algorithm"
```

---

### Task 5: should_stop Algorithm

**Files:**
- Modify: `elev_algo/src/requests.rs`

**Step 1: Write tests for should_stop**

```rust
#[test]
fn test_should_stop_cab_request_at_floor() {
    let mut e = elevator_at_floor(2);
    e.dirn = Dirn::Up;
    e.requests[2][Button::Cab.to_index()] = true;
    assert!(e.should_stop());
}

#[test]
fn test_should_stop_hall_up_when_going_up() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Up;
    e.requests[1][Button::HallUp.to_index()] = true;
    assert!(e.should_stop());
}

#[test]
fn test_should_stop_hall_down_when_going_down() {
    let mut e = elevator_at_floor(2);
    e.dirn = Dirn::Down;
    e.requests[2][Button::HallDown.to_index()] = true;
    assert!(e.should_stop());
}

#[test]
fn test_should_stop_no_more_requests_in_direction() {
    let mut e = elevator_at_floor(3);
    e.dirn = Dirn::Up;
    // No requests above floor 3, should stop
    assert!(e.should_stop());
}

#[test]
fn test_should_not_stop_requests_ahead() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Up;
    e.requests[3][Button::Cab.to_index()] = true;
    // Requests above, no request at current floor
    assert!(!e.should_stop());
}

#[test]
fn test_should_stop_when_stopped() {
    let e = elevator_at_floor(1);
    // Dirn::Stop always stops
    assert!(e.should_stop());
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL - should_stop not defined

**Step 3: Implement should_stop**

```rust
impl Elevator {
    pub fn should_stop(&self) -> bool {
        if self.floor < 0 {
            return false;
        }
        let floor = self.floor as usize;
        match self.dirn {
            Dirn::Down => {
                self.requests[floor][Button::HallDown.to_index()]
                    || self.requests[floor][Button::Cab.to_index()]
                    || !self.requests_below()
            }
            Dirn::Up => {
                self.requests[floor][Button::HallUp.to_index()]
                    || self.requests[floor][Button::Cab.to_index()]
                    || !self.requests_above()
            }
            Dirn::Stop => true,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add should_stop algorithm"
```

---

### Task 6: should_clear_immediately Algorithm

**Files:**
- Modify: `elev_algo/src/requests.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_should_clear_immediately_cab_at_floor() {
    let mut e = elevator_at_floor(2);
    e.dirn = Dirn::Up;
    assert!(e.should_clear_immediately(2, Button::Cab));
}

#[test]
fn test_should_clear_immediately_hall_up_going_up() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Up;
    assert!(e.should_clear_immediately(1, Button::HallUp));
}

#[test]
fn test_should_not_clear_immediately_hall_down_going_up() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Up;
    assert!(!e.should_clear_immediately(1, Button::HallDown));
}

#[test]
fn test_should_clear_immediately_when_stopped() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Stop;
    assert!(e.should_clear_immediately(1, Button::HallDown));
}

#[test]
fn test_should_not_clear_immediately_different_floor() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Stop;
    assert!(!e.should_clear_immediately(2, Button::Cab));
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL

**Step 3: Implement should_clear_immediately**

```rust
impl Elevator {
    pub fn should_clear_immediately(&self, btn_floor: usize, btn_type: Button) -> bool {
        self.floor == btn_floor as i32
            && (self.dirn == Dirn::Up && btn_type == Button::HallUp
                || self.dirn == Dirn::Down && btn_type == Button::HallDown
                || self.dirn == Dirn::Stop
                || btn_type == Button::Cab)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add should_clear_immediately algorithm"
```

---

### Task 7: clear_at_current_floor Algorithm

**Files:**
- Modify: `elev_algo/src/requests.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_clear_at_floor_clears_cab() {
    let mut e = elevator_at_floor(2);
    e.requests[2][Button::Cab.to_index()] = true;
    let e = e.clear_at_current_floor();
    assert!(!e.requests[2][Button::Cab.to_index()]);
}

#[test]
fn test_clear_at_floor_going_up_clears_hall_up() {
    let mut e = elevator_at_floor(1);
    e.dirn = Dirn::Up;
    e.requests[1][Button::HallUp.to_index()] = true;
    e.requests[1][Button::HallDown.to_index()] = true;
    let e = e.clear_at_current_floor();
    assert!(!e.requests[1][Button::HallUp.to_index()]);
    // HallDown should remain if there are requests above
    assert!(e.requests[1][Button::HallDown.to_index()]);
}

#[test]
fn test_clear_at_floor_going_up_clears_hall_down_if_no_requests_above() {
    let mut e = elevator_at_floor(3);
    e.dirn = Dirn::Up;
    e.requests[3][Button::HallDown.to_index()] = true;
    let e = e.clear_at_current_floor();
    // No requests above, so HallDown should be cleared too
    assert!(!e.requests[3][Button::HallDown.to_index()]);
}

#[test]
fn test_clear_at_floor_stopped_clears_both_hall() {
    let mut e = elevator_at_floor(2);
    e.dirn = Dirn::Stop;
    e.requests[2][Button::HallUp.to_index()] = true;
    e.requests[2][Button::HallDown.to_index()] = true;
    let e = e.clear_at_current_floor();
    assert!(!e.requests[2][Button::HallUp.to_index()]);
    assert!(!e.requests[2][Button::HallDown.to_index()]);
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL

**Step 3: Implement clear_at_current_floor**

```rust
impl Elevator {
    pub fn clear_at_current_floor(&self) -> Self {
        let mut e = self.clone();
        if e.floor < 0 {
            return e;
        }
        let floor = e.floor as usize;

        e.requests[floor][Button::Cab.to_index()] = false;

        match e.dirn {
            Dirn::Up => {
                if !e.requests_above() && !e.requests[floor][Button::HallUp.to_index()] {
                    e.requests[floor][Button::HallDown.to_index()] = false;
                }
                e.requests[floor][Button::HallUp.to_index()] = false;
            }
            Dirn::Down => {
                if !e.requests_below() && !e.requests[floor][Button::HallDown.to_index()] {
                    e.requests[floor][Button::HallUp.to_index()] = false;
                }
                e.requests[floor][Button::HallDown.to_index()] = false;
            }
            Dirn::Stop => {
                e.requests[floor][Button::HallUp.to_index()] = false;
                e.requests[floor][Button::HallDown.to_index()] = false;
            }
        }
        e
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add clear_at_current_floor algorithm"
```

---

### Task 8: Timer Module

**Files:**
- Create: `elev_algo/src/timer.rs`

**Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_timer_new_not_active() {
        let timer = Timer::new();
        assert!(!timer.timed_out());
    }

    #[test]
    fn test_timer_start_not_immediately_timed_out() {
        let mut timer = Timer::new();
        timer.start(1.0);
        assert!(!timer.timed_out());
    }

    #[test]
    fn test_timer_times_out() {
        let mut timer = Timer::new();
        timer.start(0.05); // 50ms
        sleep(Duration::from_millis(60));
        assert!(timer.timed_out());
    }

    #[test]
    fn test_timer_stop_prevents_timeout() {
        let mut timer = Timer::new();
        timer.start(0.05);
        timer.stop();
        sleep(Duration::from_millis(60));
        assert!(!timer.timed_out());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL

**Step 3: Implement Timer**

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Timer {
    end_time: Option<Instant>,
    duration_secs: f64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            end_time: None,
            duration_secs: 0.0,
        }
    }

    pub fn start(&mut self, duration_secs: f64) {
        self.duration_secs = duration_secs;
        self.end_time = Some(Instant::now() + std::time::Duration::from_secs_f64(duration_secs));
    }

    pub fn stop(&mut self) {
        self.end_time = None;
    }

    pub fn timed_out(&self) -> bool {
        match self.end_time {
            Some(end) => Instant::now() > end,
            None => false,
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add Timer module"
```

---

### Task 9: FSM Output Types

**Files:**
- Create: `elev_algo/src/fsm.rs`

**Step 1: Define FsmOutput struct**

```rust
use crate::{Dirn, Button};

#[derive(Debug, Clone, Default)]
pub struct FsmOutput {
    pub motor_direction: Option<Dirn>,
    pub door_light: Option<bool>,
    pub floor_indicator: Option<i32>,
    pub start_door_timer: bool,
    pub clear_lights: Vec<(usize, Button)>,
    pub set_lights: Vec<(usize, Button)>,
}

impl FsmOutput {
    pub fn new() -> Self {
        Self::default()
    }
}
```

**Step 2: Verify compiles**

Run: `cd elev_algo && cargo check`
Expected: OK

**Step 3: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add FsmOutput type"
```

---

### Task 10: FSM on_init_between_floors

**Files:**
- Modify: `elev_algo/src/fsm.rs`

**Step 1: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Elevator, Behaviour};

    #[test]
    fn test_on_init_between_floors() {
        let e = Elevator::new();
        let (e, output) = e.on_init_between_floors();
        assert_eq!(e.dirn, Dirn::Down);
        assert_eq!(e.behaviour, Behaviour::Moving);
        assert_eq!(output.motor_direction, Some(Dirn::Down));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL

**Step 3: Implement on_init_between_floors**

```rust
use crate::{Elevator, Behaviour};

impl Elevator {
    pub fn on_init_between_floors(&self) -> (Self, FsmOutput) {
        let mut e = self.clone();
        let mut output = FsmOutput::new();

        e.dirn = Dirn::Down;
        e.behaviour = Behaviour::Moving;
        output.motor_direction = Some(Dirn::Down);

        (e, output)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add on_init_between_floors FSM handler"
```

---

### Task 11: FSM on_request_button_press

**Files:**
- Modify: `elev_algo/src/fsm.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_on_request_button_press_idle_opens_door() {
    let mut e = Elevator::new();
    e.floor = 2;
    e.behaviour = Behaviour::Idle;
    e.dirn = Dirn::Stop;

    let (e, output) = e.on_request_button_press(2, Button::Cab);

    assert_eq!(e.behaviour, Behaviour::DoorOpen);
    assert_eq!(output.door_light, Some(true));
    assert!(output.start_door_timer);
}

#[test]
fn test_on_request_button_press_idle_starts_moving() {
    let mut e = Elevator::new();
    e.floor = 0;
    e.behaviour = Behaviour::Idle;
    e.dirn = Dirn::Stop;

    let (e, output) = e.on_request_button_press(3, Button::Cab);

    assert_eq!(e.behaviour, Behaviour::Moving);
    assert_eq!(e.dirn, Dirn::Up);
    assert_eq!(output.motor_direction, Some(Dirn::Up));
    assert!(e.requests[3][Button::Cab.to_index()]);
}

#[test]
fn test_on_request_button_press_moving_queues_request() {
    let mut e = Elevator::new();
    e.floor = 1;
    e.behaviour = Behaviour::Moving;
    e.dirn = Dirn::Up;

    let (e, _output) = e.on_request_button_press(3, Button::HallDown);

    assert!(e.requests[3][Button::HallDown.to_index()]);
}

#[test]
fn test_on_request_button_press_door_open_clears_immediately() {
    let mut e = Elevator::new();
    e.floor = 2;
    e.behaviour = Behaviour::DoorOpen;
    e.dirn = Dirn::Up;

    let (_e, output) = e.on_request_button_press(2, Button::HallUp);

    // Should restart timer, not queue request
    assert!(output.start_door_timer);
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL

**Step 3: Implement on_request_button_press**

```rust
impl Elevator {
    pub fn on_request_button_press(&self, btn_floor: usize, btn_type: Button) -> (Self, FsmOutput) {
        let mut e = self.clone();
        let mut output = FsmOutput::new();

        match e.behaviour {
            Behaviour::DoorOpen => {
                if e.should_clear_immediately(btn_floor, btn_type) {
                    output.start_door_timer = true;
                } else {
                    e.requests[btn_floor][btn_type.to_index()] = true;
                    output.set_lights.push((btn_floor, btn_type));
                }
            }
            Behaviour::Moving => {
                e.requests[btn_floor][btn_type.to_index()] = true;
                output.set_lights.push((btn_floor, btn_type));
            }
            Behaviour::Idle => {
                e.requests[btn_floor][btn_type.to_index()] = true;
                let pair = e.choose_direction();
                e.dirn = pair.dirn;
                e.behaviour = pair.behaviour;

                match pair.behaviour {
                    Behaviour::DoorOpen => {
                        output.door_light = Some(true);
                        output.start_door_timer = true;
                        let cleared = e.clear_at_current_floor();
                        // Collect lights to clear
                        for btn in 0..crate::N_BUTTONS {
                            if e.requests[e.floor as usize][btn] && !cleared.requests[e.floor as usize][btn] {
                                if let Some(b) = Button::from_index(btn) {
                                    output.clear_lights.push((e.floor as usize, b));
                                }
                            }
                        }
                        e = cleared;
                    }
                    Behaviour::Moving => {
                        output.motor_direction = Some(e.dirn);
                        output.set_lights.push((btn_floor, btn_type));
                    }
                    Behaviour::Idle => {
                        output.set_lights.push((btn_floor, btn_type));
                    }
                }
            }
        }

        (e, output)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add on_request_button_press FSM handler"
```

---

### Task 12: FSM on_floor_arrival

**Files:**
- Modify: `elev_algo/src/fsm.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_on_floor_arrival_updates_floor() {
    let mut e = Elevator::new();
    e.behaviour = Behaviour::Moving;
    e.dirn = Dirn::Up;

    let (e, output) = e.on_floor_arrival(2);

    assert_eq!(e.floor, 2);
    assert_eq!(output.floor_indicator, Some(2));
}

#[test]
fn test_on_floor_arrival_stops_when_should_stop() {
    let mut e = Elevator::new();
    e.behaviour = Behaviour::Moving;
    e.dirn = Dirn::Up;
    e.requests[2][Button::Cab.to_index()] = true;

    let (e, output) = e.on_floor_arrival(2);

    assert_eq!(e.behaviour, Behaviour::DoorOpen);
    assert_eq!(output.motor_direction, Some(Dirn::Stop));
    assert_eq!(output.door_light, Some(true));
    assert!(output.start_door_timer);
}

#[test]
fn test_on_floor_arrival_continues_when_should_not_stop() {
    let mut e = Elevator::new();
    e.behaviour = Behaviour::Moving;
    e.dirn = Dirn::Up;
    e.requests[3][Button::Cab.to_index()] = true;

    let (e, output) = e.on_floor_arrival(1);

    assert_eq!(e.behaviour, Behaviour::Moving);
    assert!(output.motor_direction.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL

**Step 3: Implement on_floor_arrival**

```rust
impl Elevator {
    pub fn on_floor_arrival(&self, new_floor: i32) -> (Self, FsmOutput) {
        let mut e = self.clone();
        let mut output = FsmOutput::new();

        e.floor = new_floor;
        output.floor_indicator = Some(new_floor);

        if e.behaviour == Behaviour::Moving && e.should_stop() {
            output.motor_direction = Some(Dirn::Stop);
            output.door_light = Some(true);

            let cleared = e.clear_at_current_floor();
            for btn in 0..crate::N_BUTTONS {
                if e.requests[e.floor as usize][btn] && !cleared.requests[e.floor as usize][btn] {
                    if let Some(b) = Button::from_index(btn) {
                        output.clear_lights.push((e.floor as usize, b));
                    }
                }
            }
            e = cleared;

            output.start_door_timer = true;
            e.behaviour = Behaviour::DoorOpen;
        }

        (e, output)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add on_floor_arrival FSM handler"
```

---

### Task 13: FSM on_door_timeout

**Files:**
- Modify: `elev_algo/src/fsm.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_on_door_timeout_becomes_idle() {
    let mut e = Elevator::new();
    e.floor = 2;
    e.behaviour = Behaviour::DoorOpen;
    e.dirn = Dirn::Stop;

    let (e, output) = e.on_door_timeout();

    assert_eq!(e.behaviour, Behaviour::Idle);
    assert_eq!(output.door_light, Some(false));
    assert_eq!(output.motor_direction, Some(Dirn::Stop));
}

#[test]
fn test_on_door_timeout_starts_moving() {
    let mut e = Elevator::new();
    e.floor = 1;
    e.behaviour = Behaviour::DoorOpen;
    e.dirn = Dirn::Up;
    e.requests[3][Button::Cab.to_index()] = true;

    let (e, output) = e.on_door_timeout();

    assert_eq!(e.behaviour, Behaviour::Moving);
    assert_eq!(e.dirn, Dirn::Up);
    assert_eq!(output.door_light, Some(false));
    assert_eq!(output.motor_direction, Some(Dirn::Up));
}

#[test]
fn test_on_door_timeout_stays_open_if_requests_here() {
    let mut e = Elevator::new();
    e.floor = 2;
    e.behaviour = Behaviour::DoorOpen;
    e.dirn = Dirn::Stop;
    e.requests[2][Button::Cab.to_index()] = true;

    let (e, output) = e.on_door_timeout();

    assert_eq!(e.behaviour, Behaviour::DoorOpen);
    assert!(output.start_door_timer);
}
```

**Step 2: Run test to verify it fails**

Run: `cd elev_algo && cargo test`
Expected: FAIL

**Step 3: Implement on_door_timeout**

```rust
impl Elevator {
    pub fn on_door_timeout(&self) -> (Self, FsmOutput) {
        let mut e = self.clone();
        let mut output = FsmOutput::new();

        if e.behaviour != Behaviour::DoorOpen {
            return (e, output);
        }

        let pair = e.choose_direction();
        e.dirn = pair.dirn;
        e.behaviour = pair.behaviour;

        match e.behaviour {
            Behaviour::DoorOpen => {
                output.start_door_timer = true;
                let cleared = e.clear_at_current_floor();
                for btn in 0..crate::N_BUTTONS {
                    if e.requests[e.floor as usize][btn] && !cleared.requests[e.floor as usize][btn] {
                        if let Some(b) = Button::from_index(btn) {
                            output.clear_lights.push((e.floor as usize, b));
                        }
                    }
                }
                e = cleared;
            }
            Behaviour::Moving | Behaviour::Idle => {
                output.door_light = Some(false);
                output.motor_direction = Some(e.dirn);
            }
        }

        (e, output)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd elev_algo && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): add on_door_timeout FSM handler"
```

---

### Task 14: Final Integration & Cleanup

**Files:**
- Modify: `elev_algo/src/lib.rs`

**Step 1: Verify all modules export correctly**

Run: `cd elev_algo && cargo test`
Expected: All tests PASS

**Step 2: Run clippy**

Run: `cd elev_algo && cargo clippy`
Expected: No errors (warnings OK)

**Step 3: Final commit**

```bash
git add elev_algo/
git commit -m "feat(elev_algo): complete Rust translation of elevator algorithm"
```

---

## Summary

| Task | Component | Est. Steps |
|------|-----------|------------|
| 1 | Project setup | 4 |
| 2 | Core types | 5 |
| 3 | requests_above/below/here | 5 |
| 4 | choose_direction | 5 |
| 5 | should_stop | 5 |
| 6 | should_clear_immediately | 5 |
| 7 | clear_at_current_floor | 5 |
| 8 | Timer | 5 |
| 9 | FsmOutput | 3 |
| 10 | on_init_between_floors | 5 |
| 11 | on_request_button_press | 5 |
| 12 | on_floor_arrival | 5 |
| 13 | on_door_timeout | 5 |
| 14 | Final integration | 3 |
