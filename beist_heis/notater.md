/ Run hall_request_assigner with hall_request_assigner --input "$(cat assigner_input.json)"


hall_request_assigner returns three bools per floor per node, first two are hall_requests, last are cab calls.

fn test_assigner() {
    use world_view::WorldView;
    use elev_algo::elevator::Button;
    use orders::OrderState;
    use assigner::save_assigner_input;

    let mut wv = WorldView::new(1);
    wv.set_peer_availability(1, true);
    wv.set_peer_availability(2, true);
    wv.update_order_table(1, Button::HallUp, 1, OrderState::Confirmed);
    wv.update_order_table(3, Button::HallDown, 2, OrderState::Confirmed);

    save_assigner_input(&wv, "assigner_input.json").unwrap();
    println!("saved! check assigner_input.json");
}


