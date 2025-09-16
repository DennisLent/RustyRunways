use rusty_runways_core::Game;
use rusty_runways_core::config::{AirportConfig, GameplayConfig, Location, WorldConfig};

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
        gameplay: GameplayConfig::default(),
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
        gameplay: GameplayConfig::default(),
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
        gameplay: GameplayConfig::default(),
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
        gameplay: GameplayConfig::default(),
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
        gameplay: GameplayConfig::default(),
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
        gameplay: GameplayConfig::default(),
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("runway_length"));
}

#[test]
fn from_config_applies_gameplay_tuning() {
    let mut cfg = WorldConfig {
        seed: Some(123),
        starting_cash: 1_000_000.0,
        generate_orders: true,
        airports: base_airports(),
        gameplay: GameplayConfig::default(),
    };

    cfg.gameplay.restock_cycle_hours = 72;
    cfg.gameplay.fuel_interval_hours = 8;
    cfg.gameplay.orders.max_deadline_hours = 36;
    cfg.gameplay.orders.min_weight = 500.0;
    cfg.gameplay.orders.max_weight = 750.0;
    cfg.gameplay.orders.alpha = 0.3;
    cfg.gameplay.orders.beta = 0.6;

    let game = Game::from_config(cfg).expect("should build");
    assert_eq!(game.restock_cycle, 72);
    assert_eq!(game.fuel_interval, 8);
    assert_eq!(game.map.order_params.max_deadline_hours, 36);
    assert!((game.map.order_params.min_weight - 500.0).abs() < f32::EPSILON);
    assert!((game.map.order_params.max_weight - 750.0).abs() < f32::EPSILON);
    assert!((game.map.order_params.alpha - 0.3).abs() < f32::EPSILON);
    assert!((game.map.order_params.beta - 0.6).abs() < f32::EPSILON);

    for (airport, _) in &game.map.airports {
        for order in &airport.orders {
            assert!(order.deadline <= 36);
            assert!(order.weight >= 500.0 && order.weight <= 750.0);
        }
    }
}

#[test]
fn from_config_rejects_invalid_gameplay() {
    let mut cfg = WorldConfig {
        seed: None,
        starting_cash: 1_000_000.0,
        generate_orders: false,
        airports: base_airports(),
        gameplay: GameplayConfig::default(),
    };
    cfg.gameplay.orders.min_weight = 1_000.0;
    cfg.gameplay.orders.max_weight = 100.0; // invalid

    let err = Game::from_config(cfg).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid config"));
    assert!(msg.contains("orders.max_weight"));
}
