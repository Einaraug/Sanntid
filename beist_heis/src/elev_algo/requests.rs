// Request algorithms - to be implemented

use crate::elev_algo::elevator::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirnBehaviour {
    pub dirn: Dirn,
    pub behaviour: Behaviour,
}

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

    pub fn should_clear_immediately(&self, btn_floor: usize, btn_type: Button) -> bool {
        self.floor == btn_floor as i32
            && (self.dirn == Dirn::Up && btn_type == Button::HallUp
                || self.dirn == Dirn::Down && btn_type == Button::HallDown
                || self.dirn == Dirn::Stop
                || btn_type == Button::Cab)
    }

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