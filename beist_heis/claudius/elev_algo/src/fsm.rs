use crate::{Dirn, Button, N_BUTTONS};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Elevator, Behaviour, Button};

    #[test]
    fn test_on_init_between_floors() {
        let e = Elevator::new();
        let (e, output) = e.on_init_between_floors();
        assert_eq!(e.dirn, Dirn::Down);
        assert_eq!(e.behaviour, Behaviour::Moving);
        assert_eq!(output.motor_direction, Some(Dirn::Down));
    }

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
}
