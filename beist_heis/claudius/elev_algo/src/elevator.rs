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
