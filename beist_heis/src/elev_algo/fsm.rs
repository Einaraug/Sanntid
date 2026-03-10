use crate::elev_algo::elevator::*;
use crate::elevio::elev as hw;
use crossbeam_channel as cbc;
use std::time::{Duration, Instant};

const MOTOR_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Default)]
pub struct FsmOutput {
    pub motor_direction: Option<Dirn>,
    pub door_light: Option<bool>,
    pub floor_indicator: Option<i32>,
    pub start_door_timer: bool,
    pub clear_lights: Vec<(usize, Button)>,
    pub set_lights: Vec<(usize, Button)>,
}

/// Sensor events: floor/obstruction/stop - fast path directly to FSM
pub enum SensorEvent {
    FloorArrival(u8),
    Obstruction(bool),
    StopButton(bool),
}

/// Sent FSM → WorldView when an order is completed (floor served).
pub struct CompletedOrder {
    pub floor: usize,
    pub button: Button,
}

impl FsmOutput {
    pub fn new() -> Self {
        Self::default()
    }
}


impl Elevator {
    pub fn run(
        mut self,
        hw: hw::Elevator,
        sensors: cbc::Receiver<SensorEvent>,
        orders: cbc::Receiver<[[bool; N_BUTTONS]; N_FLOORS]>,
        to_wv: cbc::Sender<Elevator>,
        to_wv_completed: cbc::Sender<CompletedOrder>,
    ) {
        let door_duration = Duration::from_secs_f64(self.door_open_duration_s);
        let mut door_timer: Option<Instant> = None;
        let mut motor_watchdog: Option<Instant> = None;
        let mut last_sent = self.clone();
        // Tracks obstruction switch state locally — does NOT propagate to WV/stuck flag,
        // so orders are never redistributed due to an obstruction alone.
        let mut obstructed = false;

        // Init: go down if between floors
        if hw.floor_sensor().is_none() {
            let (new_self, output) = self.on_init_between_floors();
            self = new_self;
            self.apply_output(&hw, &output);
        }
        // Always turn the door lamp off on startup; it may be stale from before a crash.
        hw.door_light(false);

        loop {
            // Wake up at least when the soonest timer would expire.
            let select_timeout = [door_timer, motor_watchdog]
                .into_iter()
                .flatten()
                .map(|d| d.saturating_duration_since(Instant::now()))
                .min()
                .unwrap_or(Duration::from_millis(100));

            let mut before = self.requests;

            cbc::select! {
                recv(sensors) -> msg => {
                    let Ok(event) = msg else { break };
                    let output = match event {
                        SensorEvent::FloorArrival(floor) => {
                            // Motor is working — reset watchdog
                            motor_watchdog = None;
                            let (new_self, output) = self.on_floor_arrival(floor as i32);
                            self = new_self;
                            output
                        }
                        SensorEvent::Obstruction(on) => {
                            obstructed = on;
                            if on {
                                // Block door from closing — cancel door timer.
                                // Intentionally do NOT set self.stuck here: that flag is
                                // reserved for motor failure and causes order redistribution.
                                door_timer = None;
                            } else if self.behaviour == Behaviour::DoorOpen {
                                // Obstruction cleared while door open — restart timer
                                door_timer = Some(Instant::now() + door_duration);
                            }
                            FsmOutput::new()
                        }
                        SensorEvent::StopButton(_on) => {
                            // TODO: handle stop button
                            FsmOutput::new()
                        }
                    };
                    self.apply_output(&hw, &output);
                    if output.start_door_timer {
                        door_timer = Some(Instant::now() + door_duration);
                    }
                    if matches!(output.motor_direction, Some(Dirn::Up) | Some(Dirn::Down)) {
                        motor_watchdog = Some(Instant::now() + MOTOR_TIMEOUT);
                    } else if matches!(output.motor_direction, Some(Dirn::Stop)) {
                        motor_watchdog = None;
                    }
                },
                recv(orders) -> msg => {
                    // WV pushes the full request table; trigger FSM for any newly added request.
                    let Ok(new_requests) = msg else { break };
                    for floor in 0..N_FLOORS {
                        for btn in 0..N_BUTTONS {
                            if new_requests[floor][btn] && !self.requests[floor][btn] {
                                if let Some(button) = Button::from_index(btn) {
                                    let (new_self, output) = self.on_request_button_press(floor, button);
                                    self = new_self;
                                    self.apply_output(&hw, &output);
                                    if output.start_door_timer {
                                        door_timer = Some(Instant::now() + door_duration);
                                    }
                                    if matches!(output.motor_direction, Some(Dirn::Up) | Some(Dirn::Down)) {
                                        motor_watchdog = Some(Instant::now() + MOTOR_TIMEOUT);
                                    } else if matches!(output.motor_direction, Some(Dirn::Stop)) {
                                        motor_watchdog = None;
                                    }
                                    // When a request is immediately served (elevator already at
                                    // that floor), self.requests goes false→true→false in one
                                    // step, so the before/after diff below misses it.
                                    // Use clear_lights instead, which is populated for both
                                    // Idle→DoorOpen and DoorOpen+should_clear_immediately cases.
                                    for (f, b) in &output.clear_lights {
                                        let _ = to_wv_completed.send(CompletedOrder { floor: *f, button: *b });
                                    }
                                }
                            } else if !new_requests[floor][btn] && self.requests[floor][btn] {
                                // Order was unassigned or completed by another node — drop it
                                // from FSM state without sending CompletedOrder (not served by us).
                                // Also update before so the before/after diff below doesn't
                                // mistake this for a served order and fire a spurious CompletedOrder.
                                self.requests[floor][btn] = false;
                                before[floor][btn] = false;
                            }
                        }
                    }
                },
                default(select_timeout) => {}
            }

            // Check door timer after every event, not just in the default arm.
            // Relying on default(timeout) alone fails when the orders channel is
            // continuously fed by WV, preventing default from ever firing.
            // Door timer does not fire while stuck (obstruction).
            if let Some(deadline) = door_timer {
                if !obstructed && Instant::now() >= deadline {
                    door_timer = None;
                    let (new_self, output) = self.on_door_timeout();
                    self = new_self;
                    self.apply_output(&hw, &output);
                    if output.start_door_timer {
                        door_timer = Some(Instant::now() + door_duration);
                    }
                    if matches!(output.motor_direction, Some(Dirn::Up) | Some(Dirn::Down)) {
                        motor_watchdog = Some(Instant::now() + MOTOR_TIMEOUT);
                    } else if matches!(output.motor_direction, Some(Dirn::Stop)) {
                        motor_watchdog = None;
                    }
                }
            }

            // Motor watchdog: if elevator was commanded to move but floor sensor never fired,
            // declare motor failure. Keep motor direction so recovery is automatic on power return.
            if let Some(deadline) = motor_watchdog {
                if Instant::now() >= deadline {
                    motor_watchdog = None;
                    self.stuck = true;
                }
            }

            // Notify WV of any requests cleared during this event (orders served).
            for f in 0..N_FLOORS {
                for b in 0..N_BUTTONS {
                    if before[f][b] && !self.requests[f][b] {
                        if let Some(button) = Button::from_index(b) {
                            let _ = to_wv_completed.send(CompletedOrder { floor: f, button });
                        }
                    }
                }
            }

            // Report state to WorldView only when something actually changed.
            // Sending unconditionally would create a feedback loop with WV's request-table pushes.
            if self != last_sent {
                let _ = to_wv.send(self.clone());
                last_sent = self.clone();
            }
        }
    }

    fn apply_output(&self, hw: &hw::Elevator, output: &FsmOutput) {
        if let Some(dir) = output.motor_direction {
            hw.motor_direction(match dir {
                Dirn::Up   => hw::DIRN_UP,
                Dirn::Down => hw::DIRN_DOWN,
                Dirn::Stop => hw::DIRN_STOP,
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
    pub fn on_init_between_floors(&self) -> (Self, FsmOutput) {
        let mut e = self.clone();
        let mut output = FsmOutput::new();

        e.dirn = Dirn::Down;
        e.behaviour = Behaviour::Moving;
        output.motor_direction = Some(Dirn::Down);

        (e, output)
    }

    pub fn on_request_button_press(&self, btn_floor: usize, btn_type: Button) -> (Self, FsmOutput) {
        let mut e = self.clone();
        let mut output = FsmOutput::new();

        match e.behaviour {
            Behaviour::DoorOpen => {
                if e.should_clear_immediately(btn_floor, btn_type) {
                    output.start_door_timer = true;
                    output.clear_lights.push((btn_floor, btn_type));
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
                        for btn in 0..N_BUTTONS {
                            if e.requests[e.floor as usize][btn]
                                && !cleared.requests[e.floor as usize][btn]
                            {
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

    pub fn on_floor_arrival(&self, new_floor: i32) -> (Self, FsmOutput) {
        let mut e = self.clone();
        let mut output = FsmOutput::new();

        e.floor = new_floor;
        output.floor_indicator = Some(new_floor);

        // Recovering from motor failure: clear stuck and fall through to normal logic.
        // The elevator is still Moving, so should_stop() will handle stopping at the
        // right floor. Forcing Idle here would strand any remaining requests.
        e.stuck = false;

        if e.behaviour == Behaviour::Moving && e.should_stop() {
            output.motor_direction = Some(Dirn::Stop);
            output.door_light = Some(true);

            let cleared = e.clear_at_current_floor();
            for btn in 0..N_BUTTONS {
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
                for btn in 0..N_BUTTONS {
                    if e.requests[e.floor as usize][btn] && !cleared.requests[e.floor as usize][btn]
                    {
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
