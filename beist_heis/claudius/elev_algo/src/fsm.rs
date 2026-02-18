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
