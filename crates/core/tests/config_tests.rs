use rusty_runways_core::Game;
use rusty_runways_core::config::{AirportConfig, Location, WorldConfig};

fn base_airports() -> Vec<AirportConfig> {
    vec![
        AirportConfig {
            id: 0,
            name: "AAA".into(),
            location: Location {
                x: 1000.0,
                y: 1000.0,
            },
            runway_length_m: 3000.0,
            fuel_price_per_l: 1.2,
            landing_fee_per_ton: 5.0,
            parking_fee_per_hour: 20.0,
        },
        AirportConfig {
            id: 1,
            name: "AAB".into(),
            location: Location {
                x: 2000.0,
                y: 1500.0,
            },
            runway_length_m: 2500.0,
            fuel_price_per_l: 1.8,
            landing_fee_per_ton: 4.5,
            parking_fee_per_hour: 15.0,
        },
    ]
}

#[test]
fn from_config_generates_orders_when_enabled() {
    let cfg = WorldConfig {
        seed: Some(1),
        starting_cash: 1_000_000.0,
        generate_orders: true,
        airports: base_airports(),
    };
    let game = Game::from_config(cfg).expect("should build");
    // both airports should have non-empty orders generally
    let any_orders = game.map.airports.iter().any(|(a, _)| !a.orders.is_empty());
    assert!(any_orders, "expected some orders to be generated");
}

#[test]
fn from_config_no_orders_when_disabled() {
    let cfg = WorldConfig {
        seed: Some(1),
        starting_cash: 1_000_000.0,
        generate_orders: false,
        airports: base_airports(),
    };
    let game = Game::from_config(cfg).expect("should build");
    assert!(game.map.airports.iter().all(|(a, _)| a.orders.is_empty()));
}

#[test]
fn from_config_duplicate_ids_is_error() {
    let mut airports = base_airports();
    airports[1].id = airports[0].id; // duplicate
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 1_000_000.0,
        generate_orders: false,
        airports,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(
        format!("{}", err)
            .to_lowercase()
            .contains("duplicate airport id")
    );
}

#[test]
fn from_config_duplicate_names_is_error() {
    let mut airports = base_airports();
    airports[1].name = airports[0].name.clone();
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 1_000_000.0,
        generate_orders: false,
        airports,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(
        format!("{}", err)
            .to_lowercase()
            .contains("duplicate airport name")
    );
}

#[test]
fn from_config_location_bounds_enforced() {
    let mut airports = base_airports();
    airports[1].location.x = 20000.0; // out of bounds
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 1_000_000.0,
        generate_orders: false,
        airports,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("out of bounds"));
}

#[test]
fn from_config_positive_values_required() {
    let mut airports = base_airports();
    airports[0].runway_length_m = 0.0;
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 1_000_000.0,
        generate_orders: false,
        airports,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("runway_length"));
}
