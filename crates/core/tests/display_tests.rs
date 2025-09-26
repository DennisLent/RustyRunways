#![cfg(feature = "ui_prints")]

use rusty_runways_core::game::Game;
use rusty_runways_core::utils::airplanes::models::AirplaneStatus;
use rusty_runways_core::utils::errors::GameError;
use rusty_runways_core::utils::orders::order::{Order, OrderPayload};

#[test]
fn show_and_list_helpers_execute() {
    let mut game = Game::new(11, Some(6), 750_000.0);

    game.show_cash();
    game.show_time();
    game.show_stats();

    game.list_airports(true);
    game.list_airports(false);
    game.list_airport(0, true).unwrap();
    // Insert a passenger order to exercise formatting branch
    game.map.airports[0].0.orders.push(Order {
        id: 999,
        payload: OrderPayload::Passengers { count: 12 },
        value: 7_500.0,
        deadline: 10,
        origin_id: 0,
        destination_id: 1,
    });
    game.list_airports(true);
    game.list_airplane(0).unwrap();
    game.list_airplanes().unwrap();
    game.show_distances(0).unwrap();

    // Attempt to fetch invalid airport triggers error path
    let err = game.list_airport(usize::MAX, true).unwrap_err();
    assert!(matches!(err, GameError::AirportIdInvalid { .. }));

    let plane_id = 0usize;
    let origin_loc = game.airplanes[plane_id].location;
    let origin_idx = game
        .airports()
        .iter()
        .position(|(_, c)| *c == origin_loc)
        .expect("origin airport must exist");
    let payload_cap = game.airplanes[plane_id].specs.payload_capacity;

    let mut candidate: Option<(usize, usize)> = None;
    for _ in 0..5 {
        for order in &game.map.airports[origin_idx].0.orders {
            let Some(weight) = order.cargo_weight() else {
                continue;
            };
            if weight <= payload_cap {
                let (airport, coord) = &game.airports()[order.destination_id];
                if game.planes()[plane_id].can_fly_to(airport, coord).is_ok() {
                    candidate = Some((order.id, order.destination_id));
                    break;
                }
            }
        }
        if candidate.is_some() {
            break;
        }
        game.map.restock_airports();
    }

    let (order_id, dest_idx) = candidate.expect("expected reachable order");
    game.load_order(order_id, plane_id).unwrap();
    game.advance(1);
    game.depart_plane(plane_id, dest_idx).unwrap();
    assert!(matches!(
        game.airplanes[plane_id].status,
        AirplaneStatus::InTransit { .. }
    ));

    game.list_airplane(plane_id).unwrap();
    game.show_distances(plane_id).unwrap();
}
