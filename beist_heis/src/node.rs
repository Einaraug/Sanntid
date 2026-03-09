use crate::elev_algo::elevator::{Button, Elevator, N_FLOORS, N_BUTTONS};
use crate::elev_algo::fsm::CompletedOrder;
use crate::elevio::elev as hw;
use crate::elevio::poll::ButtonEvent;
use crate::orders::{OrderState, OrderTable, UNASSIGNED_NODE};
use crate::world_view::{WorldView, N_NODES};
use crate::counters;
use crossbeam_channel as cbc;
use std::time::{Duration, Instant};

const PEER_TIMEOUT:       Duration = Duration::from_millis(500);
const BROADCAST_INTERVAL: Duration = Duration::from_millis(100);

pub fn run(
    mut wv: WorldView,
    hw: hw::Elevator,
    from_buttons:       cbc::Receiver<ButtonEvent>,
    from_network:       cbc::Receiver<WorldView>,
    from_fsm_state:     cbc::Receiver<Elevator>,
    from_fsm_completed: cbc::Receiver<CompletedOrder>,
    from_assigner:      cbc::Receiver<OrderTable>,
    to_fsm:             cbc::Sender<[[bool; N_BUTTONS]; N_FLOORS]>,
    to_network:         cbc::Sender<WorldView>,
    to_assigner:        cbc::Sender<WorldView>,
) {
    let mut last_broadcast     = Instant::now();
    let mut last_peer_check    = Instant::now();
    let mut last_sent_requests = [[false; N_BUTTONS]; N_FLOORS];

    loop {
        cbc::select! {
            recv(from_buttons) -> msg => {
                let Ok(btn) = msg else { break };
                if let Some(button) = Button::from_index(btn.button as usize) {
                    let changes = wv.order_table.on_button_press(btn.floor as usize, button, wv.self_id);
                    wv.counters.apply(changes);
                }
            },

            recv(from_fsm_state) -> msg => {
                let Ok(elev) = msg else { break };
                let was_stuck = wv.elevator_map.get(wv.self_id).stuck;
                if elev != *wv.elevator_map.get(wv.self_id) {
                    let changes = wv.elevator_map.set(wv.self_id, elev);
                    wv.counters.apply(changes);
                }
                // Newly stuck: unassign our orders so other elevators pick them up
                if elev.stuck && !was_stuck {
                    let changes = wv.order_table.unassign_orders_for(wv.self_id);
                    wv.counters.apply(changes);
                }
            },

            recv(from_fsm_completed) -> msg => {
                let Ok(completed) = msg else { break };
                let changes = match completed.button {
                    Button::HallUp | Button::HallDown =>
                        wv.order_table.clear_hall_order(completed.floor, completed.button),
                    Button::Cab =>
                        wv.order_table.clear_cab_order(completed.floor, wv.self_id),
                };
                wv.counters.apply(changes);
            },

            recv(from_network) -> msg => {
                let Ok(peer_wv) = msg else { break };
                if peer_wv.self_id != wv.self_id {
                    let changes = wv.peer_monitor.mark_seen(peer_wv.self_id, PEER_TIMEOUT);
                    wv.counters.apply(changes);
                    counters::merge(&mut wv, &peer_wv);
                }
            },

            recv(from_assigner) -> msg => {
                let Ok(assigned) = msg else { break };
                for floor in 0..N_FLOORS {
                    for btn in [Button::HallUp, Button::HallDown] {
                        let a = assigned.get_hall_order(floor, btn.to_index());
                        let c = wv.order_table.get_hall_order(floor, btn.to_index());
                        if a.node_id == wv.self_id
                            && c.state == OrderState::Confirmed
                            && c.node_id == UNASSIGNED_NODE
                        {
                            let changes = wv.order_table.assign_node_id(floor, btn, a.node_id);
                            wv.counters.apply(changes);
                        }
                    }
                }
            },

            default(BROADCAST_INTERVAL) => {}
        }

        let changes = wv.order_table.try_confirm_orders(&wv.peer_monitor.availability);
        wv.counters.apply(changes);

        if last_peer_check.elapsed() >= PEER_TIMEOUT {
            let (dead, changes) = wv.peer_monitor.expire_stale_peers();
            wv.counters.apply(changes);
            for node_id in dead {
                let changes = wv.order_table.unassign_orders_for(node_id);
                wv.counters.apply(changes);
            }
            last_peer_check = Instant::now();
        }

        update_lights(&wv, &hw);

        let requests = wv.order_table.convert_to_requests(wv.self_id);
        if requests != last_sent_requests {
            let _ = to_fsm.send(requests);
            last_sent_requests = requests;
        }

        let _ = to_assigner.try_send(wv.clone());

        if last_broadcast.elapsed() >= BROADCAST_INTERVAL {
            let _ = to_network.send(wv.clone());
            last_broadcast = Instant::now();
        }
    }
}

fn update_lights(wv: &WorldView, hw: &hw::Elevator) {
    for floor in 0..N_FLOORS {
        for btn in [Button::HallUp, Button::HallDown] {
            let on = wv.order_table.get_hall_order(floor, btn.to_index()).state == OrderState::Confirmed;
            hw.call_button_light(floor as u8, btn.to_index() as u8, on);
        }
        let on = wv.order_table.get_cab_order(floor, wv.self_id).state == OrderState::Confirmed;
        hw.call_button_light(floor as u8, Button::Cab.to_index() as u8, on);
    }
}
