use serde::{Serialize, Deserialize};

pub const N_FLOORS: usize = 4;
pub const N_BUTTONS: usize = 3;
pub const DOOR_OPEN_DURATION: f64 = 2.0;

/// Motor direction commanded to the hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Dirn {
    Down = -1,
    Stop = 0,
    Up = 1,
}

/// Which button was pressed — determines order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Button {
    HallUp,
    HallDown,
    Cab,
}

impl Button {
    pub fn to_index(self) -> usize {
        match self {
            Button::HallUp   => 0,
            Button::HallDown => 1,
            Button::Cab      => 2,
        }
    }

    pub fn from_index(btn_id: usize) -> Option<Self> {
        match btn_id {
            0 => Some(Button::HallUp),
            1 => Some(Button::HallDown),
            2 => Some(Button::Cab),
            _ => None,
        }
    }
}

/// FSM state of the elevator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Behaviour {
    #[default]
    Idle,
    DoorOpen,
    Moving,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Elevator {
    pub floor: i32,
    pub dirn: Dirn,
    pub requests: [[bool; N_BUTTONS]; N_FLOORS],
    pub behaviour: Behaviour,
    pub door_open_duration_s: f64,
    pub stuck: bool,
}

impl Elevator {
    pub fn new() -> Self {
        Self {
            floor: 0,
            dirn: Dirn::Stop,
            requests: [[false; N_BUTTONS]; N_FLOORS],
            behaviour: Behaviour::Idle,
            door_open_duration_s: DOOR_OPEN_DURATION,
            stuck: false,
        }
    }
}