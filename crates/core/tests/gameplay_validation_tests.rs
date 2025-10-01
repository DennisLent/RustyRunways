use rusty_runways_core::Game;
use rusty_runways_core::config::{
    AirportConfig, GameplayConfig, Location, ManualOrderConfig, WorldConfig,
};
use rusty_runways_core::utils::orders::cargo::CargoType;

fn base_airports() -> Vec<AirportConfig> {
    vec![
        AirportConfig {
            id: 0,
            name: "HUB".into(),
            location: Some(Location {
                x: 1200.0,
                y: 900.0,
            }),
            runway_length_m: Some(3200.0),
            fuel_price_per_l: Some(1.5),
            landing_fee_per_ton: Some(4.5),
            parking_fee_per_hour: Some(18.0),
            orders: Vec::new(),
        },
        AirportConfig {
            id: 1,
            name: "AAX".into(),
            location: Some(Location {
                x: 3400.0,
                y: 2100.0,
            }),
            runway_length_m: Some(2400.0),
            fuel_price_per_l: Some(1.8),
            landing_fee_per_ton: Some(4.0),
            parking_fee_per_hour: Some(16.0),
            orders: Vec::new(),
        },
    ]
}

fn cfg_with_airports() -> WorldConfig {
    WorldConfig {
        seed: Some(1),
        starting_cash: 650_000.0,
        airports: base_airports(),
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    }
}

#[test]
fn gameplay_validation_branches() {
    // fuel_interval_hours == 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.fuel_interval_hours = 0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("fuel_interval_hours"));

    // orders tuning: max_deadline_hours == 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.orders.tuning.max_deadline_hours = 0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("orders.max_deadline_hours"));

    // orders tuning: min_weight <= 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.orders.tuning.min_weight = 0.0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("orders.min_weight"));

    // passenger tuning: max_deadline_hours == 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.orders.passengers.max_deadline_hours = 0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("passengers.max_deadline_hours"));

    // passenger tuning: min_count == 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.orders.passengers.min_count = 0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("passengers.min_count"));

    // passenger tuning: max_count < min_count
    let mut cfg = cfg_with_airports();
    cfg.gameplay.orders.passengers.min_count = 50;
    cfg.gameplay.orders.passengers.max_count = 10;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("passengers.max_count"));

    // passenger tuning: fare_per_km <= 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.orders.passengers.fare_per_km = 0.0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("fare_per_km"));

    // fuel.elasticity <= 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.fuel.elasticity = 0.0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("fuel.elasticity"));

    // fuel.elasticity >= 1
    let mut cfg = cfg_with_airports();
    cfg.gameplay.fuel.elasticity = 1.0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("fuel.elasticity"));

    // min_price_multiplier <= 0
    let mut cfg = cfg_with_airports();
    cfg.gameplay.fuel.min_price_multiplier = 0.0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("min_price_multiplier"));

    // max_price_multiplier < min_price_multiplier
    let mut cfg = cfg_with_airports();
    cfg.gameplay.fuel.min_price_multiplier = 1.0;
    cfg.gameplay.fuel.max_price_multiplier = 0.9;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("max_price_multiplier"));

    // max_price_multiplier <= 1
    let mut cfg = cfg_with_airports();
    cfg.gameplay.fuel.max_price_multiplier = 1.0;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("max_price_multiplier must be greater than 1"));
}

#[test]
fn manual_order_validation_errors() {
    // Regeneration disabled requires manual orders that are valid; we feed one invalid at a time
    // Cargo with deadline == 0
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Cargo {
        cargo: CargoType::Food,
        weight: 100.0,
        value: 1000.0,
        deadline_hours: 0,
        destination_id: 1,
    }];
    let mut cfg = cfg_with_airports();
    cfg.airports = airports.clone();
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("deadline_hours == 0"));

    // Cargo weight <= 0
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Cargo {
        cargo: CargoType::Food,
        weight: 0.0,
        value: 1000.0,
        deadline_hours: 24,
        destination_id: 1,
    }];
    let mut cfg = cfg_with_airports();
    cfg.airports = airports;
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("non-positive weight"));

    // Cargo negative value
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Cargo {
        cargo: CargoType::Food,
        weight: 10.0,
        value: -1.0,
        deadline_hours: 24,
        destination_id: 1,
    }];
    let mut cfg = cfg_with_airports();
    cfg.airports = airports;
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("negative value"));

    // Cargo destination self
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Cargo {
        cargo: CargoType::Food,
        weight: 10.0,
        value: 1.0,
        deadline_hours: 24,
        destination_id: 0,
    }];
    let mut cfg = cfg_with_airports();
    cfg.airports = airports;
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("pointing to itself"));

    // Passenger order with zero passengers
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Passengers {
        passengers: 0,
        value: 1.0,
        deadline_hours: 24,
        destination_id: 1,
    }];
    let mut cfg = cfg_with_airports();
    cfg.airports = airports;
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("zero passengers"));

    // Passenger destination unknown
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Passengers {
        passengers: 10,
        value: 10.0,
        deadline_hours: 24,
        destination_id: 999, // not in airports list
    }];
    let mut cfg = cfg_with_airports();
    cfg.airports = airports;
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("unknown destination"));
}
