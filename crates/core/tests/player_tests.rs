use rusty_runways_core::player::Player;
use rusty_runways_core::utils::airport::Airport;
use rusty_runways_core::utils::coordinate::Coordinate;
use rusty_runways_core::utils::errors::GameError;
use rusty_runways_core::utils::map::Map;

#[test]
fn player_initialization_gives_single_plane() {
    let map = Map::generate_from_seed(7, Some(4));
    let player = Player::new(650_000.0, &map);
    assert_eq!(player.fleet_size, 1);
    assert_eq!(player.fleet.len(), 1);
    assert_eq!(player.orders_delivered, 0);

    let (_min_distance, start_index) = map.min_distance();
    let expected = map.airports[start_index].1;
    let plane = &player.fleet[0];
    assert_eq!(plane.location.x, expected.x);
    assert_eq!(plane.location.y, expected.y);

    // chosen model must have range over min_distance
    let model_range = plane.max_range();
    let (distance, _) = map.min_distance();
    assert!(model_range > distance);
}

#[test]
fn buy_plane_success() {
    let map = Map::generate_from_seed(1, Some(2));
    let mut player = Player::new(650_000.0, &map);
    let mut airport = Airport::generate_random(1, 10);
    airport.runway_length = 3000.0;
    let coord = Coordinate::new(0.0, 0.0);
    assert!(
        player
            .buy_plane(&"SparrowLight".to_string(), &mut airport, &coord)
            .is_ok()
    );
    assert_eq!(player.fleet_size, 2);
}

#[test]
fn buy_plane_unknown_model() {
    let map = Map::generate_from_seed(2, Some(2));
    let mut player = Player::new(650_000.0, &map);
    let mut airport = Airport::generate_random(2, 0);
    let coord = Coordinate::new(0.0, 0.0);
    let result = player.buy_plane(&"NotAPlane".to_string(), &mut airport, &coord);
    assert!(matches!(result, Err(GameError::UnknownModel { .. })));
}

#[test]
fn buy_plane_insufficient_funds() {
    let map = Map::generate_from_seed(3, Some(2));
    let mut player = Player::new(10.0, &map);
    let mut airport = Airport::generate_random(3, 0);
    airport.runway_length = 5000.0;
    let coord = Coordinate::new(0.0, 0.0);
    let result = player.buy_plane(&"SparrowLight".to_string(), &mut airport, &coord);
    assert!(matches!(result, Err(GameError::InsufficientFunds { .. })));
}

#[test]
fn buy_plane_runway_too_short() {
    let map = Map::generate_from_seed(4, Some(2));
    let mut player = Player::new(650_000.0, &map);
    let mut airport = Airport::generate_random(4, 0);
    airport.runway_length = 100.0;
    let coord = Coordinate::new(0.0, 0.0);
    let result = player.buy_plane(&"SparrowLight".to_string(), &mut airport, &coord);
    assert!(matches!(result, Err(GameError::RunwayTooShort { .. })));
}

#[test]
fn record_delivery_increments_counter() {
    let map = Map::generate_from_seed(5, Some(2));
    let mut player = Player::new(650_000.0, &map);
    assert_eq!(player.orders_delivered, 0);
    player.record_delivery();
    player.record_delivery();
    assert_eq!(player.orders_delivered, 2);
}

#[test]
fn buy_plane_deducts_cash() {
    use rusty_runways_core::utils::airplanes::models::AirplaneModel;
    let map = Map::generate_from_seed(6, Some(2));
    let mut player = Player::new(650_000.0, &map);
    let mut airport = Airport::generate_random(6, 10);
    airport.runway_length = 4000.0;
    let coord = Coordinate::new(0.0, 0.0);
    let price = AirplaneModel::SparrowLight.specs().purchase_price;
    player
        .buy_plane(&"SparrowLight".to_string(), &mut airport, &coord)
        .unwrap();
    assert!((player.cash - (650_000.0 - price)).abs() < f32::EPSILON);
}

#[test]
fn sell_plane_returns_refund_and_updates_state() {
    let map = Map::generate_from_seed(7, Some(3));
    let mut player = Player::new(650_000.0, &map);
    let starting_cash = player.cash;
    let (plane, refund) = player.sell_plane(0).expect("plane 0 should exist");
    assert_eq!(plane.id, 0);
    assert!((refund - plane.specs.purchase_price * 0.6).abs() < f32::EPSILON);
    assert!((player.cash - (starting_cash + refund)).abs() < f32::EPSILON);
    assert_eq!(player.fleet_size, player.fleet.len());
    assert!(player.fleet.iter().all(|p| p.id != 0));
}
