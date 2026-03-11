use crate::elev_algo::elevator::*;
use crate::elev_algo::timer::Timer;
use crate::elevio::elev as hw;
use crossbeam_channel as cbc;
use std::time::Duration;

const MOTOR_TIMEOUT_SECS: f64 = 4.0;

/// Extracts Ok(val) from a channel recv result, or breaks the loop on disconnect.
macro_rules! unwrap_or_break {
    ($msg:expr) => {
        match $msg {
            Ok(val) => val,
            Err(_)  => break,
        }
    };
}

#[derive(Debug, Clone, Default)]
pub struct FsmOutput {
    pub motor_direction: Option<Dirn>,
    pub door_light: Option<bool>,
    pub floor_indicator: Option<i32>,
    pub start_door_timer: bool,
    pub clear_lights: Vec<(usize, Button)>,
    pub set_lights: Vec<(usize, Button)>,
    pub completed_orders: Vec<(usize, Button)>,
}

pub enum SensorEvent {
    FloorArrival(u8),
    Obstruction(bool),
    StopButton(bool),
}

/// Messagetype to be sent from FSM to WV
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
        to_node: cbc::Sender<Elevator>,
        to_node_completed: cbc::Sender<CompletedOrder>,
    ) 
    {
        let door_open_duration_s = self.door_open_duration_s;
        let mut door_timer = Timer::new();
        let mut motor_watchdog = Timer::new();
        let mut last_sent = self.clone();
        let mut obstructed = false;

        if hw.floor_sensor().is_none() {
            let (new_self, output) = self.on_init_between_floors();
            self = new_self;
            self.apply_output(&hw, &output);
        }
        // Always turn the door lamp off on startup; it may be stale from before a crash.
        hw.door_light(false);

        loop {
            let select_timeout = Self::time_until_next_deadline(&door_timer, &motor_watchdog);
            let mut before = self.requests;

            // Handle incoming event
            cbc::select! {
                recv(sensors) -> msg => {
                    let event = unwrap_or_break!(msg);
                    let output = match event {
                        SensorEvent::FloorArrival(floor) => {
                            motor_watchdog.cancel();
                            let (new_self, output) = self.on_floor_arrival(floor as i32);
                            self = new_self;
                            output
                        }
                        SensorEvent::Obstruction(on) => {
                            obstructed = on;
                            if on {
                                // stuck is reserved for motor failure (triggers order
                                // redistribution). Obstruction only blocks the door timer.
                                door_timer.cancel();
                            } else if self.behaviour == Behaviour::DoorOpen {
                                door_timer.start(door_open_duration_s);
                            }
                            FsmOutput::new()
                        }
                        SensorEvent::StopButton(_on) => {
                            // Ignore stop_button
                            FsmOutput::new()
                        }
                    };
                    
                    self.apply_output(&hw, &output);
                    Self::update_timers(&output, &mut door_timer, &mut motor_watchdog, door_open_duration_s);
                },
                recv(orders) -> msg => {
                    let new_requests = unwrap_or_break!(msg);

                    for floor in 0..N_FLOORS {
                        for btn in 0..N_BUTTONS {
                            if new_requests[floor][btn] && !self.requests[floor][btn] {
                                let button = Button::from_index(btn).unwrap();
                                let (new_self, output) = self.on_request_button_press(floor, button);
                                self = new_self;
                                self.apply_output(&hw, &output);
                                Self::update_timers(&output, &mut door_timer, &mut motor_watchdog, door_open_duration_s);
                                // If served immediately, requests goes false→true→false in one
                                // step and the diff below misses it — report via completed_orders.
                                for (floor, btn) in &output.completed_orders {
                                    let _ = to_node_completed.send(CompletedOrder {floor: *floor, button: *btn});
                                }
                            } else if !new_requests[floor][btn] && self.requests[floor][btn] {
                                // Order dropped externally — clear without reporting as completed,
                                // and sync before[] to suppress a spurious diff entry.
                                self.requests[floor][btn] = false;
                                before[floor][btn] = false;
                            }
                        }
                    }
                },
                default(select_timeout) => {}
            }

            // Door timer
            // Evaluated after every arm — a busy orders channel starves default.
            if !obstructed && door_timer.timed_out() {
                door_timer.cancel();
                let (new_self, output) = self.on_door_timeout();
                self = new_self;
                self.apply_output(&hw, &output);
                Self::update_timers(&output, &mut door_timer, &mut motor_watchdog, door_open_duration_s);
            }

            // Motor watchdog
            if motor_watchdog.timed_out() {
                motor_watchdog.cancel();
                self.stuck = true;
            }

            for floor in 0..N_FLOORS {
                for btn in 0..N_BUTTONS {
                    if before[floor][btn] && !self.requests[floor][btn] {
                        let _ = to_node_completed.send(CompletedOrder{floor, button: Button::from_index(btn).unwrap()});
                    }
                }
            }

            if self != last_sent {
                let _ = to_node.send(self.clone());
                last_sent = self.clone();
            }
        }
    }

    fn update_timers(
        output: &FsmOutput,
        door_timer: &mut Timer,
        motor_watchdog: &mut Timer,
        door_open_duration_s: f64,
    ) {
        if output.start_door_timer {
            door_timer.start(door_open_duration_s);
        }
        if matches!(output.motor_direction, Some(Dirn::Up) | Some(Dirn::Down)) {
            motor_watchdog.start(MOTOR_TIMEOUT_SECS);
        } else if matches!(output.motor_direction, Some(Dirn::Stop)) {
            motor_watchdog.cancel();
        }
    }

    /// Returns how long the select should block before waking to check timers.
    /// Picks the soonest active deadline, falling back to 100 ms if none are set.
    fn time_until_next_deadline(door_timer: &Timer, motor_watchdog: &Timer) -> Duration {
        [door_timer.remaining(), motor_watchdog.remaining()]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(Duration::from_millis(100))
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
                    output.completed_orders.push((btn_floor, btn_type));
                } 
                else {
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
                        for btn in 0..N_BUTTONS {
                            if e.requests[e.floor as usize][btn]
                                && !cleared.requests[e.floor as usize][btn]
                            {
                                let b = Button::from_index(btn).unwrap();
                                output.clear_lights.push((e.floor as usize, b));
                                output.completed_orders.push((e.floor as usize, b));
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
