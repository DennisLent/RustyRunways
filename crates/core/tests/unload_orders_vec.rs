use rusty_runways_core::game::Game;
use rusty_runways_core::utils::airplanes::models::AirplaneStatus;

// Helper to locate a reachable order from the plane's current airport
fn reachable_order(game: &Game, plane_id: usize) -> Option<(usize, usize, f32)> {
    let origin_loc = game.planes()[plane_id].location;
    let origin_idx = game.airports().iter().position(|(_, c)| *c == origin_loc)?;
    let payload_cap = game.planes()[plane_id].specs.payload_capacity;
    game.airports().get(origin_idx).and_then(|(airport, _)| {
        airport.orders.iter().find_map(|order| {
            let weight = order.cargo_weight()?;
            let plane = &game.planes()[plane_id];
            let (a, c) = &game.airports()[order.destination_id];
            if weight <= payload_cap && plane.can_fly_to(a, c).is_ok() {
                Some((order.id, order.destination_id, order.value))
            } else {
                None
            }
        })
    })
}

#[test]
fn unload_orders_vector_works_for_single_item() {
    let mut game = Game::new(2, Some(6), 10_000_000.0);
    let plane_id = 0usize;

    // restock a few times until we can find a candidate
    let (order_id, dest_idx, value) = {
        let mut found = None;
        for _ in 0..10 {
            if let Some(c) = reachable_order(&game, plane_id) {
                found = Some(c);
                break;
            }
            game.map.restock_airports();
        }
        found.expect("expected at least one reachable order")
    };

    game.load_order(order_id, plane_id).unwrap();
    game.advance(1);
    assert!(matches!(
        game.planes()[plane_id].status,
        AirplaneStatus::Parked
    ));

    // fly to destination
    game.depart_plane(plane_id, dest_idx).unwrap();
    let hours = match game.planes()[plane_id].status {
        AirplaneStatus::InTransit {
            hours_remaining, ..
        } => hours_remaining,
        _ => unreachable!(),
    };
    game.advance(hours);
    assert!(matches!(
        game.planes()[plane_id].status,
        AirplaneStatus::Parked
    ));

    let income_before = game.daily_income;
    game.unload_orders(vec![order_id], plane_id).unwrap();
    game.advance(1);
    assert!(game.daily_income >= income_before + value - 1e-3);
}
