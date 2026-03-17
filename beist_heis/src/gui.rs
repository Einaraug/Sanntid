use crossbeam_channel as cbc;
use std::thread;
use std::time::Duration;
use crate::world_view::{WorldView, N_NODES};
use crate::elev_algo::elevator::{N_FLOORS, Button};
use crate::orders::OrderState;

const REFRESH: Duration = Duration::from_millis(200);

pub fn run(rx: cbc::Receiver<WorldView>) {
    let mut latest: Option<WorldView> = None;
    loop {
        while let Ok(wv) = rx.try_recv() {
            latest = Some(wv);
        }
        if let Some(ref wv) = latest {
            render(wv);
        }
        thread::sleep(REFRESH);
    }
}

fn render(wv: &WorldView) {
    print!("\x1b[2J\x1b[H");

    // ── Header ────────────────────────────────────────────────────────
    print!("NODE {}   peers:", wv.self_id);
    for id in 0..N_NODES {
        let sym = if wv.peer_monitor.availability[id] { "●" } else { "○" };
        print!("  {}:{}", id, sym);
    }
    println!("\n");

    // ── Elevator states ───────────────────────────────────────────────
    println!("ELEVATOR STATES");
    println!("  {:>4}  {:>5}  {:<8}  {:<10}  {}", "node", "floor", "dir", "behaviour", "stuck");
    for id in 0..N_NODES {
        let e = wv.node_states.get(id);
        let offline = if id != wv.self_id && !wv.peer_monitor.availability[id] { " (offline)" } else { "" };
        println!("  {:>4}  {:>5}  {:<8?}  {:<10?}  {}{}", id, e.floor, e.dirn, e.behaviour, e.stuck, offline);
    }

    // ── Hall orders ───────────────────────────────────────────────────
    println!("\nHALL ORDERS");
    println!("  {:>5}  {:<18}  {:<18}", "floor", "up", "down");
    for floor in (0..N_FLOORS).rev() {
        let up   = fmt_hall(&wv.order_table.hall[floor][Button::HallUp.to_index()]);
        let down = fmt_hall(&wv.order_table.hall[floor][Button::HallDown.to_index()]);
        println!("  {:>5}  {:<18}  {:<18}", floor, up, down);
    }

    // ── Cab orders ────────────────────────────────────────────────────
    print!("\nCAB ORDERS    ");
    for id in 0..N_NODES { print!("  node{}", id); }
    println!();
    for floor in (0..N_FLOORS).rev() {
        print!("  floor {}      ", floor);
        for id in 0..N_NODES {
            let s = match wv.order_table.cab[floor][id].state {
                OrderState::None        => "  -   ",
                OrderState::Unconfirmed => "  u   ",
                OrderState::Confirmed   => "  C   ",
            };
            print!("{}", s);
        }
        println!();
    }

    // ── Counters ──────────────────────────────────────────────────────
    println!("\nCOUNTERS");
    print!("  elevator:  ");
    for id in 0..N_NODES { print!("  {}:{}", id, wv.counters.get_elevator(id)); }
    println!();
    print!("  peer avail:");
    for id in 0..N_NODES { print!("  {}:{}", id, wv.counters.get_peer_availability(id)); }
    println!();
    println!("  hall orders:");
    for floor in (0..N_FLOORS).rev() {
        print!("    floor {}:", floor);
        for btn in [Button::HallUp, Button::HallDown] {
            let label = if btn == Button::HallUp { "up" } else { "dn" };
            print!("  {}:{}", label, wv.counters.get_hall_order(floor, btn));
        }
        println!();
    }
    print!("  cab orders:");
    for id in 0..N_NODES { print!("  {}:{}", id, (0..N_FLOORS).map(|f| wv.counters.get_cab_order(f, id)).sum::<u64>()); }
    println!();
}

fn fmt_hall(order: &crate::orders::HallOrder) -> String {
    match order.state {
        OrderState::None        => "-".to_string(),
        OrderState::Unconfirmed => "unconf".to_string(),
        OrderState::Confirmed if order.assigned_to.is_none() => "conf (unassigned)".to_string(),
        OrderState::Confirmed   => format!("conf → {}", order.assigned_to.unwrap()),
    }
}
