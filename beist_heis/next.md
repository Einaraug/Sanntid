#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::elev_algo::elevator::{Elevator, Dirn, Behaviour, N_BUTTONS, N_FLOORS};
    use crate::orders::OrderState;

    #[test]
    fn test_build_input_json() {
        let mut elevators = HashMap::new();
        elevators.insert(1u32, Elevator {
            floor: 2,
            dirn: Dirn::Up,
            behaviour: Behaviour::Moving,
            requests: [[false; N_BUTTONS]; N_FLOORS],
            door_open_duration_s: 3.0,
        });
        elevators.insert(2u32, Elevator {
            floor: 0,
            dirn: Dirn::Stop,
            behaviour: Behaviour::Idle,
            requests: [[false; N_BUTTONS]; N_FLOORS],
            door_open_duration_s: 3.0,
        });

        let mut peer_availability = HashMap::new();
        peer_availability.insert(1u32, true);
        peer_availability.insert(2u32, true);

        let mut hall = [[OrderState::None, OrderState::None]; N_FLOORS];
        hall[1][0] = OrderState::Confirmed;
        hall[3][1] = OrderState::Confirmed;

        let wv = WorldView {
            // fill in fields using your actual WorldView constructor or struct literal
        };

        let input = build_input(&wv);
        let json = serde_json::to_string_pretty(&input).unwrap();
        std::fs::write("test_input.json", json).unwrap();
    }
}
