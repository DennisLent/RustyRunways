use rusty_runways_core::Game;
use rusty_runways_core::config::{
    AirplaneCatalogStrategy, AirplaneModelConfig, AirplanesConfig, GameplayConfig, Location,
    WorldConfig,
};

fn airport(id: usize, name: &str, x: f32, y: f32) -> rusty_runways_core::config::AirportConfig {
    rusty_runways_core::config::AirportConfig {
        id,
        name: name.to_string(),
        location: Some(Location { x, y }),
        runway_length_m: Some(4000.0),
        fuel_price_per_l: Some(1.2),
        landing_fee_per_ton: Some(4.0),
        parking_fee_per_hour: Some(10.0),
        orders: Vec::new(),
    }
}

fn base_model(name: &str, price: f32) -> AirplaneModelConfig {
    AirplaneModelConfig {
        name: name.to_string(),
        mtow: 10_000.0,
        cruise_speed: 400.0,
        fuel_capacity: 2_000.0,
        fuel_consumption: 200.0,
        operating_cost: 500.0,
        payload_capacity: 1_000.0,
        passenger_capacity: 0,
        purchase_price: price,
        min_runway_length: 800.0,
        role: rusty_runways_core::utils::airplanes::models::AirplaneRole::Cargo,
    }
}

#[test]
fn from_config_uses_replace_catalog_for_starter() {
    let airports = vec![
        airport(0, "HUB", 1000.0, 1000.0),
        airport(1, "AAX", 1200.0, 1005.0),
    ];
    let airplanes = AirplanesConfig {
        strategy: AirplaneCatalogStrategy::Replace,
        models: vec![
            base_model("CustomCheap", 100_000.0),
            base_model("CustomExpensive", 2_000_000.0),
        ],
    };
    let cfg = WorldConfig {
        seed: Some(7),
        starting_cash: 1_000_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: Some(airplanes),
    };

    let game = Game::from_config(cfg).expect("config should build");
    assert_eq!(game.planes()[0].specs.purchase_price, 100_000.0);
}

#[test]
fn from_config_adds_catalog_and_picks_cheapest() {
    let airports = vec![
        airport(0, "AAA", 1000.0, 1000.0),
        airport(1, "BBB", 1200.0, 1005.0),
    ];
    let airplanes = AirplanesConfig {
        strategy: AirplaneCatalogStrategy::Add,
        models: vec![base_model("UltraCheap", 50_000.0)],
    };
    let cfg = WorldConfig {
        seed: Some(8),
        starting_cash: 1_000_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: Some(airplanes),
    };

    let game = Game::from_config(cfg).expect("config should build");
    assert_eq!(game.planes()[0].specs.purchase_price, 50_000.0);
}

#[test]
fn replace_disallows_unknown_model_purchase() {
    let airports = vec![
        airport(0, "AAA", 1000.0, 1000.0),
        airport(1, "BBB", 1200.0, 1005.0),
    ];
    let airplanes = AirplanesConfig {
        strategy: AirplaneCatalogStrategy::Replace,
        models: vec![base_model("OnlyModel", 120_000.0)],
    };
    let cfg = WorldConfig {
        seed: Some(9),
        starting_cash: 2_000_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: Some(airplanes),
    };
    let mut game = Game::from_config(cfg.clone()).expect("should build");
    // buying default should fail in replace mode
    let e = game.buy_plane(&"SparrowLight".to_string(), 0).unwrap_err();
    if let rusty_runways_core::utils::errors::GameError::UnknownModel { .. } = e {
    } else {
        panic!("expected UnknownModel in replace mode");
    }
    // buying the custom model should succeed
    game.buy_plane(&"OnlyModel".to_string(), 0)
        .expect("buy custom");
}

#[test]
fn invalid_model_rejected() {
    let airports = vec![
        airport(0, "AAA", 1000.0, 1000.0),
        airport(1, "BBB", 1200.0, 1005.0),
    ];
    let mut bad = base_model("BadOne", 100_000.0);
    bad.purchase_price = -1.0;
    let airplanes = AirplanesConfig {
        strategy: AirplaneCatalogStrategy::Replace,
        models: vec![bad],
    };
    let cfg = WorldConfig {
        seed: Some(5),
        starting_cash: 1_000_000.0,
        airports,
        num_airports: None,
        gameplay: GameplayConfig::default(),
        airplanes: Some(airplanes),
    };
    let err = Game::from_config(cfg).unwrap_err();
    if let rusty_runways_core::utils::errors::GameError::InvalidConfig { msg } = err {
        assert!(msg.contains("non-positive"));
    } else {
        panic!("expected invalid config error");
    }
}
