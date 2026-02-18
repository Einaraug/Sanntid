// Request algorithms - to be implemented

use crate::elevator::{Elevator, N_FLOORS, N_BUTTONS, Dirn, Behaviour, Button};

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

#[cfg(test)]
mod tests {
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

    #[test]
    fn test_should_stop_cab_request_at_floor() {
        let mut e = elevator_at_floor(2);
        e.dirn = Dirn::Up;
        e.requests[2][Button::Cab.to_index()] = true;
        assert!(e.should_stop());
    }

    #[test]
    fn test_should_stop_hall_up_when_going_up() {
        let mut e = elevator_at_floor(1);
        e.dirn = Dirn::Up;
        e.requests[1][Button::HallUp.to_index()] = true;
        assert!(e.should_stop());
    }

    #[test]
    fn test_should_stop_hall_down_when_going_down() {
        let mut e = elevator_at_floor(2);
        e.dirn = Dirn::Down;
        e.requests[2][Button::HallDown.to_index()] = true;
        assert!(e.should_stop());
    }

    #[test]
    fn test_should_stop_no_more_requests_in_direction() {
        let mut e = elevator_at_floor(3);
        e.dirn = Dirn::Up;
        // No requests above floor 3, should stop
        assert!(e.should_stop());
    }

    #[test]
    fn test_should_not_stop_requests_ahead() {
        let mut e = elevator_at_floor(1);
        e.dirn = Dirn::Up;
        e.requests[3][Button::Cab.to_index()] = true;
        // Requests above, no request at current floor
        assert!(!e.should_stop());
    }

    #[test]
    fn test_should_stop_when_stopped() {
        let e = elevator_at_floor(1);
        // Dirn::Stop always stops
        assert!(e.should_stop());
    }

    #[test]
    fn test_should_clear_immediately_cab_at_floor() {
        let mut e = elevator_at_floor(2);
        e.dirn = Dirn::Up;
        assert!(e.should_clear_immediately(2, Button::Cab));
    }

    #[test]
    fn test_should_clear_immediately_hall_up_going_up() {
        let mut e = elevator_at_floor(1);
        e.dirn = Dirn::Up;
        assert!(e.should_clear_immediately(1, Button::HallUp));
    }

    #[test]
    fn test_should_not_clear_immediately_hall_down_going_up() {
        let mut e = elevator_at_floor(1);
        e.dirn = Dirn::Up;
        assert!(!e.should_clear_immediately(1, Button::HallDown));
    }

    #[test]
    fn test_should_clear_immediately_when_stopped() {
        let mut e = elevator_at_floor(1);
        e.dirn = Dirn::Stop;
        assert!(e.should_clear_immediately(1, Button::HallDown));
    }

    #[test]
    fn test_should_not_clear_immediately_different_floor() {
        let mut e = elevator_at_floor(1);
        e.dirn = Dirn::Stop;
        assert!(!e.should_clear_immediately(2, Button::Cab));
    }

    #[test]
    fn test_clear_at_floor_clears_cab() {
        let mut e = elevator_at_floor(2);
        e.requests[2][Button::Cab.to_index()] = true;
        let e = e.clear_at_current_floor();
        assert!(!e.requests[2][Button::Cab.to_index()]);
    }

    #[test]
    fn test_clear_at_floor_going_up_clears_hall_up() {
        let mut e = elevator_at_floor(1);
        e.dirn = Dirn::Up;
        e.requests[1][Button::HallUp.to_index()] = true;
        e.requests[1][Button::HallDown.to_index()] = true;
        e.requests[3][Button::Cab.to_index()] = true;  // Request above to keep HallDown
        let e = e.clear_at_current_floor();
        assert!(!e.requests[1][Button::HallUp.to_index()]);
        // HallDown should remain if there are requests above
        assert!(e.requests[1][Button::HallDown.to_index()]);
    }

    #[test]
    fn test_clear_at_floor_going_up_clears_hall_down_if_no_requests_above() {
        let mut e = elevator_at_floor(3);
        e.dirn = Dirn::Up;
        e.requests[3][Button::HallDown.to_index()] = true;
        let e = e.clear_at_current_floor();
        // No requests above, so HallDown should be cleared too
        assert!(!e.requests[3][Button::HallDown.to_index()]);
    }

    #[test]
    fn test_clear_at_floor_stopped_clears_both_hall() {
        let mut e = elevator_at_floor(2);
        e.dirn = Dirn::Stop;
        e.requests[2][Button::HallUp.to_index()] = true;
        e.requests[2][Button::HallDown.to_index()] = true;
        let e = e.clear_at_current_floor();
        assert!(!e.requests[2][Button::HallUp.to_index()]);
        assert!(!e.requests[2][Button::HallDown.to_index()]);
    }
}
