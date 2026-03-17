use crate::elev_algo::elevator::{Button, Elevator, N_FLOORS, N_BUTTONS};
use crate::elev_algo::fsm::CompletedOrder;
use crate::elevio::poll::ButtonEvent;
use crate::orders::{OrderState, OrderTable};
use crate::world_view::WorldView;
use crossbeam_channel as cbc;
use std::time::{Duration, Instant};

const BROADCAST_INTERVAL: Duration = Duration::from_millis(100);
const PEER_TIMEOUT_INTERVAL: Duration = Duration::from_millis(500);

macro_rules! unwrap_or_break {
    ($msg:expr) => {
        match $msg {
            Ok(val) => val,
            Err(_)  => break,
        }
    };
}

pub fn run(
    from_buttons: cbc::Receiver<ButtonEvent>,
    from_network: cbc::Receiver<WorldView>,
    from_fsm_state: cbc::Receiver<Elevator>,
    from_fsm_completed: cbc::Receiver<CompletedOrder>,
    from_assigner: cbc::Receiver<OrderTable>,
    mut wv: WorldView,
    to_fsm: cbc::Sender<[[bool; N_BUTTONS]; N_FLOORS]>,
    to_network: cbc::Sender<WorldView>,
    to_assigner: cbc::Sender<WorldView>,
    to_lights: cbc::Sender<OrderTable>,
)
{
    let mut last_broadcast = Instant::now(); 
    let mut last_peer_check = Instant::now(); 
    let mut last_sent_requests = [[false; N_BUTTONS]; N_FLOORS];

    loop {
        cbc::select! {
            recv(from_buttons) -> msg => {
                let btn = unwrap_or_break!(msg);

                if let Some(button) = Button::from_index(btn.button as usize) {
                    let changes = wv.order_table.on_btn_press(btn.floor as usize, button, wv.self_id);
                    wv.counters.apply(changes);
                }
            },

            recv(from_fsm_state) -> msg => {
                let node_state = unwrap_or_break!(msg);

                let was_stuck = wv.node_states.get(wv.self_id).stuck;
                if node_state != wv.node_states.get(wv.self_id) {
                    let changes = wv.node_states.set(wv.self_id, node_state);
                    wv.counters.apply(changes);
                }
                
                if node_state.stuck && !was_stuck {
                    let changes = wv.order_table.unassign_orders_for(wv.self_id);
                    wv.counters.apply(changes);
                }
            },

            recv(from_fsm_completed) -> msg => {
                let completed_order = unwrap_or_break!(msg);

                let changes = match completed_order.button {
                    Button::HallUp | Button::HallDown =>
                        wv.order_table.clear_hall_order(completed_order.floor, completed_order.button),
                    Button::Cab =>
                        wv.order_table.clear_cab_order(completed_order.floor, wv.self_id),
                };
                wv.counters.apply(changes);
            },

            recv(from_network) -> msg => {
                let peer_wv = unwrap_or_break!(msg);

                if peer_wv.self_id != wv.self_id {
                    let changes = wv.peer_monitor.mark_seen(peer_wv.self_id);
                    wv.counters.apply(changes);
                    wv.merge_from(&peer_wv);
                }
            },

            recv(from_assigner) -> msg => {
                let assigned = unwrap_or_break!(msg);

                for floor in 0..N_FLOORS {
                    for btn in [Button::HallUp, Button::HallDown] {
                        let suggested_order = assigned.get_hall_order(floor, btn.to_index());
                        let current_order   = wv.order_table.get_hall_order(floor, btn.to_index());
                        if should_assign(&suggested_order, &current_order, &wv) {
                            let changes = wv.order_table.assign_order_to(floor, btn, wv.self_id);
                            wv.counters.apply(changes);
                        }
                    }
                }
            },

            default(BROADCAST_INTERVAL) => {} 
        }
        let changes = wv.order_table.try_confirm_orders(&wv.peer_monitor.availability);
        wv.counters.apply(changes);
        
        if last_peer_check.elapsed() >= PEER_TIMEOUT_INTERVAL {
            let (stale_peers, changes) = wv.peer_monitor.expire_stale_peers();
            wv.counters.apply(changes);
            
            for node_id in stale_peers {
                let changes = wv.order_table.unassign_orders_for(node_id);
                wv.counters.apply(changes);
            }
            last_peer_check = Instant::now();
        }

        let requests = wv.order_table.build_fsm_request_table(wv.self_id);
        if requests != last_sent_requests {
            let _ = to_fsm.send(requests);
            last_sent_requests = requests;
        }
        
        let _ = to_assigner.try_send(wv.clone());
        let _ = to_lights.try_send(wv.order_table.clone());

        if last_broadcast.elapsed() >= BROADCAST_INTERVAL {
            let _ = to_network.send(wv.clone());
            last_broadcast = Instant::now();
        }
    }
}

fn should_assign(suggested: &crate::orders::HallOrder, current: &crate::orders::HallOrder, wv: &WorldView) -> bool {
    suggested.assigned_to == Some(wv.self_id)
        && current.assigned_to.is_none()
        && current.state == OrderState::Confirmed
        && !wv.node_states.get(wv.self_id).stuck
}

