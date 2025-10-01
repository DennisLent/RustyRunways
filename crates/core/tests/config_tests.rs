use rusty_runways_core::Game;
use rusty_runways_core::config::{
    AirportConfig, GameplayConfig, Location, ManualOrderConfig, WorldConfig,
};
use rusty_runways_core::utils::orders::cargo::CargoType;

fn base_airports() -> Vec<AirportConfig> {
    vec![
        AirportConfig {
            id: 0,
            name: "AAA".into(),
            location: Some(Location {
                x: 1000.0,
                y: 1000.0,
            }),
            runway_length_m: Some(3000.0),
            fuel_price_per_l: Some(1.2),
            landing_fee_per_ton: Some(5.0),
            parking_fee_per_hour: Some(20.0),
            orders: Vec::new(),
        },
        AirportConfig {
            id: 1,
            name: "AAB".into(),
            location: Some(Location {
                x: 2000.0,
                y: 1500.0,
            }),
            runway_length_m: Some(2500.0),
            fuel_price_per_l: Some(1.8),
            landing_fee_per_ton: Some(4.5),
            parking_fee_per_hour: Some(15.0),
            orders: Vec::new(),
        },
    ]
}

#[test]
fn from_config_generates_orders_when_enabled() {
    let cfg = WorldConfig {
        seed: Some(1),
        starting_cash: 650_000.0,
        airports: base_airports(),
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    let game = Game::from_config(cfg).expect("should build");
    // both airports should have non-empty orders generally
    let any_orders = game.map.airports.iter().any(|(a, _)| !a.orders.is_empty());
    assert!(any_orders, "expected some orders to be generated");
}

#[test]
fn from_config_no_orders_when_disabled() {
    let mut cfg = WorldConfig {
        seed: Some(1),
        starting_cash: 650_000.0,
        airports: base_airports(),
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    cfg.gameplay.orders.generate_initial = false;
    let game = Game::from_config(cfg).expect("should build");
    assert!(game.map.airports.iter().all(|(a, _)| a.orders.is_empty()));
}

#[test]
fn from_config_duplicate_ids_is_error() {
    let mut airports = base_airports();
    airports[1].id = airports[0].id; // duplicate
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 650_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
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
        starting_cash: 650_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
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
    airports[1].location = Some(Location {
        x: 20_000.0,
        y: 1_500.0,
    });
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 650_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("out of bounds"));
}

#[test]
fn from_config_positive_values_required() {
    let mut airports = base_airports();
    airports[0].runway_length_m = Some(0.0);
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 650_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).to_lowercase().contains("runway_length"));
}

#[test]
fn from_config_applies_gameplay_tuning() {
    let mut cfg = WorldConfig {
        seed: Some(123),
        starting_cash: 650_000.0,
        airports: base_airports(),
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };

    cfg.gameplay.restock_cycle_hours = 72;
    cfg.gameplay.fuel_interval_hours = 8;
    cfg.gameplay.orders.generate_initial = false;
    cfg.gameplay.orders.tuning.max_deadline_hours = 36;
    cfg.gameplay.orders.tuning.min_weight = 500.0;
    cfg.gameplay.orders.tuning.max_weight = 750.0;
    cfg.gameplay.orders.tuning.alpha = 0.3;
    cfg.gameplay.orders.tuning.beta = 0.6;
    cfg.gameplay.fuel.elasticity = 0.04;
    cfg.gameplay.fuel.min_price_multiplier = 0.6;
    cfg.gameplay.fuel.max_price_multiplier = 1.4;

    let game = Game::from_config(cfg).expect("should build");
    assert_eq!(game.restock_cycle, 72);
    assert_eq!(game.fuel_interval, 8);
    assert_eq!(game.map.demand_params.cargo.max_deadline_hours, 36);
    assert!((game.map.demand_params.cargo.min_weight - 500.0).abs() < f32::EPSILON);
    assert!((game.map.demand_params.cargo.max_weight - 750.0).abs() < f32::EPSILON);
    assert!((game.map.demand_params.cargo.alpha - 0.3).abs() < f32::EPSILON);
    assert!((game.map.demand_params.cargo.beta - 0.6).abs() < f32::EPSILON);
    assert!((game.fuel_settings.elasticity - 0.04).abs() < f32::EPSILON);
    assert!((game.fuel_settings.min_price_multiplier - 0.6).abs() < f32::EPSILON);
    assert!((game.fuel_settings.max_price_multiplier - 1.4).abs() < f32::EPSILON);
    // generate_initial disabled => airports start empty even though regeneration remains enabled
    assert!(game.map.airports.iter().all(|(a, _)| a.orders.is_empty()));
}

#[test]
fn from_config_rejects_invalid_gameplay() {
    let mut cfg = WorldConfig {
        seed: None,
        starting_cash: 650_000.0,
        airports: base_airports(),
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    cfg.gameplay.orders.tuning.min_weight = 1_000.0;
    cfg.gameplay.orders.tuning.max_weight = 100.0; // invalid

    let err = Game::from_config(cfg).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid config"));
    assert!(msg.contains("orders.max_weight"));
}

#[test]
fn from_config_generated_airports_when_requested() {
    let cfg = WorldConfig {
        seed: Some(17),
        starting_cash: 750_000.0,
        airports: Vec::new(),
        num_airports: Some(4),
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    let game = Game::from_config(cfg).expect("should build");
    assert_eq!(game.map.num_airports, 4);
}

#[test]
fn from_config_requires_num_airports_when_none_provided() {
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 500_000.0,
        airports: Vec::new(),
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("num_airports"));
}

#[test]
fn from_config_rejects_num_airports_with_explicit_airports() {
    let cfg = WorldConfig {
        seed: None,
        starting_cash: 500_000.0,
        airports: base_airports(),
        num_airports: Some(2),
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    let err = Game::from_config(cfg).unwrap_err();
    assert!(format!("{}", err).contains("num_airports"));
}

#[test]
fn from_config_regeneration_disabled_requires_orders() {
    let mut cfg = WorldConfig {
        seed: Some(0),
        starting_cash: 650_000.0,
        airports: base_airports(),
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;

    let err = Game::from_config(cfg).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("must define at least one order"));
}

#[test]
fn from_config_uses_manual_orders_when_regeneration_disabled() {
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Cargo {
        cargo: CargoType::Food,
        weight: 500.0,
        value: 2_500.0,
        deadline_hours: 48,
        destination_id: 1,
    }];
    airports[1].orders = vec![ManualOrderConfig::Cargo {
        cargo: CargoType::Electronics,
        weight: 300.0,
        value: 4_200.0,
        deadline_hours: 24,
        destination_id: 0,
    }];

    let mut cfg = WorldConfig {
        seed: Some(9),
        starting_cash: 650_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;

    let game = Game::from_config(cfg).expect("should build");
    assert!(!game.regenerate_orders);
    let airport_orders: Vec<_> = game
        .map
        .airports
        .iter()
        .map(|(a, _)| a.orders.len())
        .collect();
    assert_eq!(airport_orders, vec![1, 1]);
    let ids: Vec<_> = game
        .map
        .airports
        .iter()
        .flat_map(|(a, _)| a.orders.iter().map(|o| o.id))
        .collect();
    assert_eq!(ids, vec![0, 1]);
}

#[test]
fn from_config_supports_manual_passengers() {
    let mut airports = base_airports();
    airports[0].orders = vec![ManualOrderConfig::Passengers {
        passengers: 18,
        value: 6_500.0,
        deadline_hours: 24,
        destination_id: 1,
    }];
    airports[1].orders = vec![ManualOrderConfig::Cargo {
        cargo: CargoType::Food,
        weight: 250.0,
        value: 1_800.0,
        deadline_hours: 30,
        destination_id: 0,
    }];

    let mut cfg = WorldConfig {
        seed: Some(21),
        starting_cash: 500_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };
    cfg.gameplay.orders.regenerate = false;
    cfg.gameplay.orders.generate_initial = false;

    let game = Game::from_config(cfg).expect("should build");
    let airport_orders = &game.map.airports[0].0.orders;
    assert_eq!(airport_orders.len(), 1);
    let order = &airport_orders[0];
    assert!(order.is_passenger());
    assert_eq!(order.passenger_count(), Some(18));
    assert_eq!(order.value, 6_500.0);
}

#[test]
fn from_config_generates_missing_fields_and_locations() {
    let airports = vec![
        AirportConfig {
            id: 0,
            name: "Hub".into(),
            location: None,
            runway_length_m: None,
            fuel_price_per_l: None,
            landing_fee_per_ton: None,
            parking_fee_per_hour: None,
            orders: Vec::new(),
        },
        AirportConfig {
            id: 1,
            name: "Spoke".into(),
            location: None,
            runway_length_m: Some(2_400.0),
            fuel_price_per_l: Some(1.6),
            landing_fee_per_ton: None,
            parking_fee_per_hour: None,
            orders: Vec::new(),
        },
    ];
    let cfg = WorldConfig {
        seed: Some(88),
        starting_cash: 750_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: None,
    };

    let game = Game::from_config(cfg).expect("should build");
    assert_eq!(game.map.airports.len(), 2);

    let (first, coord0) = &game.map.airports[0];
    assert!(coord0.x >= 0.0 && coord0.x <= 10_000.0);
    assert!(coord0.y >= 0.0 && coord0.y <= 10_000.0);
    assert!(first.runway_length > 0.0);
    assert!(first.fuel_price > 0.0);

    let (second, _) = &game.map.airports[1];
    assert_eq!(second.fuel_price, 1.6);
    assert_eq!(second.runway_length, 2_400.0);
}
