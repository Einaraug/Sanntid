// Request algorithms - to be implemented

use crate::elevator::{Elevator, N_FLOORS, N_BUTTONS, Dirn, Behaviour};

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
}

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
}
