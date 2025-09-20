use rusty_runways_core::{
    game::Game,
    utils::{
        airplanes::models::{AirplaneModel, AirplaneStatus},
        errors::GameError,
    },
};
#[test]
fn test_game_new() {
    let game = Game::new(1, Some(5), 650_000.0);
    assert_eq!(game.map.num_airports, 5);
    assert_eq!(game.player.cash, 650_000.0);
}

#[test]
fn buy_new_plane() {
    let mut game = Game::new(1, Some(5), 10_000_000.0);

    // before
    let cash_before = game.player.cash;
    assert_eq!(cash_before, 10_000_000.0);
    assert_eq!(game.player.fleet_size, 1);

    // Shouldn't fail
    game.buy_plane(&"FalconJet".to_string(), 0).unwrap();

    let cash_after = game.player.cash;
    assert_eq!(
        cash_after,
        cash_before - AirplaneModel::FalconJet.specs().purchase_price
    );
    assert_eq!(game.player.fleet_size, 2);

    let new_plane = game.airplanes.last().unwrap();
    let target_airport = game.map.airports[0].1;
    assert_eq!(new_plane.location, target_airport);
}

#[test]
fn load_check() {
    let mut game = Game::new(1, Some(5), 10_000_000.0);

    let plane_id = 0usize;
    let origin_loc = game.airplanes[plane_id].location;
    let origin_idx = game
        .airports()
        .iter()
        .position(|(_, c)| *c == origin_loc)
        .expect("origin airport should exist");
    let payload_cap = game.airplanes[plane_id].specs.payload_capacity;

    let mut heavy_id: Option<usize> = None;

    for _ in 0..4 {
        for order in &game.map.airports[origin_idx].0.orders {
            let Some(weight) = order.cargo_weight() else {
                continue;
            };
            if weight > payload_cap && heavy_id.is_none() {
                heavy_id = Some(order.id);
            }
        }
        if heavy_id.is_some() {
            break;
        }
        game.map.restock_airports();
    }

    if let Some(id) = heavy_id {
        assert!(matches!(
            game.load_order(id, plane_id),
            Err(GameError::MaxPayloadReached { .. })
        ));
        assert_eq!(game.airplanes[plane_id].status, AirplaneStatus::Parked);
    }

    let mut candidate: Option<(usize, usize)> = None;
    for _ in 0..5 {
        candidate = game.map.airports.get(origin_idx).and_then(|(airport, _)| {
            airport.orders.iter().find_map(|order| {
                let weight = order.cargo_weight()?;
                let plane = &game.planes()[plane_id];
                let (airport_dest, coord_dest) = &game.airports()[order.destination_id];
                if weight <= payload_cap && plane.can_fly_to(airport_dest, coord_dest).is_ok() {
                    Some((order.id, order.destination_id))
                } else {
                    None
                }
            })
        });
        if candidate.is_some() {
            break;
        }
        game.map.restock_airports();
    }

    let (light_id, _dest_idx) = candidate.expect("expected at least one loadable order");
    game.load_order(light_id, plane_id).unwrap();
    assert_eq!(game.airplanes[plane_id].status, AirplaneStatus::Loading);

    game.advance(1);

    assert_eq!(game.airplanes[plane_id].status, AirplaneStatus::Parked);
    assert!(game.airplanes[plane_id].current_payload <= payload_cap);
    assert_eq!(game.airplanes[plane_id].manifest.len(), 1);
    assert_eq!(game.airplanes[plane_id].manifest[0].id, light_id);
}

#[test]
fn delivery_cycle() {
    let mut game = Game::new(1, Some(5), 10_000_000.0);

    let plane_id = 0usize;
    let origin_loc = game.airplanes[plane_id].location;
    let origin_idx = game
        .airports()
        .iter()
        .position(|(_, c)| *c == origin_loc)
        .expect("origin airport should exist");
    let payload_cap = game.airplanes[plane_id].specs.payload_capacity;

    let mut candidate: Option<(usize, usize, f32)> = None;
    for _ in 0..5 {
        candidate = game.map.airports.get(origin_idx).and_then(|(airport, _)| {
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
        });
        if candidate.is_some() {
            break;
        }
        game.map.restock_airports();
    }

    let (order_id, dest_idx, order_value) = candidate.expect("expected reachable order");
    game.load_order(order_id, plane_id).unwrap();
    game.advance(1);

    let before_takeoff = game.player.cash;
    game.depart_plane(plane_id, dest_idx).unwrap();
    assert!(matches!(
        game.airplanes[plane_id].status,
        AirplaneStatus::InTransit { .. }
    ));
    assert!(game.player.cash < before_takeoff);

    let (hours_remaining, _) = match game.airplanes[plane_id].status {
        AirplaneStatus::InTransit {
            hours_remaining,
            destination,
            ..
        } => (hours_remaining, destination),
        _ => unreachable!(),
    };

    let before_landing = game.player.cash;
    game.advance(hours_remaining);
    assert_eq!(game.airplanes[plane_id].status, AirplaneStatus::Parked);
    assert_eq!(game.arrival_times.get(&plane_id), Some(&game.time));
    assert_eq!(
        game.airplanes[plane_id].location,
        game.map.airports[dest_idx].1
    );
    let landing_fee =
        (game.airplanes[plane_id].specs.mtow / 1000.0) * game.map.airports[dest_idx].0.landing_fee;
    assert!((before_landing - game.player.cash - landing_fee).abs() < 1.0);

    let before_unload = game.player.cash;
    game.unload_all(plane_id).unwrap();
    assert_eq!(game.airplanes[plane_id].status, AirplaneStatus::Unloading);

    game.advance(1);
    assert_eq!(game.airplanes[plane_id].status, AirplaneStatus::Parked);
    assert!(game.airplanes[plane_id].manifest.is_empty());
    assert_eq!(game.airplanes[plane_id].current_payload, 0.0);
    assert!((game.player.cash - before_unload - order_value).abs() < 1.0);

    let cash_before_refuel = game.player.cash;
    let fueling_fee = (game.airplanes[plane_id].specs.fuel_capacity
        - game.airplanes[plane_id].current_fuel)
        * game.map.airports[dest_idx].0.fuel_price;
    game.refuel_plane(plane_id).unwrap();
    assert_eq!(game.airplanes[plane_id].status, AirplaneStatus::Refueling);

    game.advance(1);
    assert!((game.player.cash - (cash_before_refuel - fueling_fee)).abs() < 1.0);
    assert_eq!(
        game.airplanes[plane_id].current_fuel,
        game.airplanes[plane_id].specs.fuel_capacity
    );
}
