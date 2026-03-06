use std::usize;

use serde::{Serialize, Deserialize};

pub const N_FLOORS: usize = 4;

pub const N_BUTTONS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
#[serde(rename_all = "camelCase")]
pub enum Dirn {
    Down = -1,
    Stop = 0,
    Up = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}

impl Elevator {
    pub fn new() -> Self {
        Self {
            floor: 0,
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