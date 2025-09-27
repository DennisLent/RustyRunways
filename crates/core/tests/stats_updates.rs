use rusty_runways_core::{game::Game, utils::airplanes::models::AirplaneStatus};

// Helper: find a reachable cargo order for plane 0 at its current airport
fn find_reachable_order_mut(game: &mut Game, plane_id: usize) -> (usize, usize, f32) {
    for _ in 0..12 {
        let origin_loc = game.planes()[plane_id].location;
        let origin_idx = game
            .airports()
            .iter()
            .position(|(_, c)| *c == origin_loc)
            .expect("origin must exist");
        let payload_cap = game.planes()[plane_id].specs.payload_capacity;

        if let Some((order_id, dest_idx, value)) =
            game.airports().get(origin_idx).and_then(|(airport, _)| {
                airport.orders.iter().find_map(|order| {
                    let weight = order.cargo_weight()?;
                    let plane = &game.planes()[plane_id];
                    let (airport_dest, coord_dest) = &game.airports()[order.destination_id];
                    if weight <= payload_cap && plane.can_fly_to(airport_dest, coord_dest).is_ok() {
                        Some((order.id, order.destination_id, order.value))
                    } else {
                        None
                    }
                })
            })
        {
            return (order_id, dest_idx, value);
        }
        game.map.restock_airports();
    }
    panic!("failed to find reachable order after restocking");
}

#[test]
fn live_income_expenses_and_deliveries_update() {
    let mut game = Game::new(42, Some(6), 1_000_000.0);

    let plane_id = 0usize;
    // Find a loadable and reachable order
    let (order_id, dest_idx, order_value) = find_reachable_order_mut(&mut game, plane_id);

    // Load and finish loading tick
    game.load_order(order_id, plane_id).unwrap();
    game.advance(1);
    assert!(matches!(
        game.planes()[plane_id].status,
        AirplaneStatus::Parked
    ));

    // Track starting counters
    let delivered_before = game.player.orders_delivered;
    let income_before = game.daily_income;
    let expenses_before = game.daily_expenses;

    // Depart and finish flight; landing should add a landing fee to daily_expenses
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
    assert!(
        game.daily_expenses > expenses_before,
        "landing fee should increase expenses"
    );

    // Unload and finish unloading tick; income and deliveries should increase
    game.unload_all(plane_id).unwrap();
    game.advance(1);
    assert!(game.daily_income >= income_before + order_value - 1e-3);
    assert_eq!(game.player.orders_delivered, delivered_before + 1);

    // Refuel and finish fueling tick; expenses should increase by at least some positive fee
    let expenses_pre_refuel = game.daily_expenses;
    game.refuel_plane(plane_id).unwrap();
    game.advance(1);
    assert!(game.daily_expenses > expenses_pre_refuel);
}
