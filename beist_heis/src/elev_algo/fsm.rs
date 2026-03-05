use crate::elev_algo::elevator::*;
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

impl Elevator {
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
