use rusty_runways_core::Game;
use rusty_runways_core::config::{
    AirportConfig, FuelGameplay, GameplayConfig, Location, ManualOrderConfig, OrderTuning,
    OrdersGameplay, PassengerTuning, WorldConfig,
};
use rusty_runways_core::events::{Event, ScheduledEvent};
use rusty_runways_core::utils::airplanes::models::AirplaneStatus;
use rusty_runways_core::utils::orders::order::OrderPayload;
use rusty_runways_core::utils::orders::{CargoType, Order};
use tempfile::tempdir;

fn base_gameplay() -> GameplayConfig {
    GameplayConfig {
        restock_cycle_hours: 168,
        fuel_interval_hours: 6,
        orders: OrdersGameplay {
            regenerate: true,
            generate_initial: true,
            tuning: OrderTuning::default(),
            passengers: PassengerTuning::default(),
        },
        fuel: FuelGameplay::default(),
    }
}

fn airport(
    id: usize,
    name: &str,
    location: Option<Location>,
    orders: Vec<ManualOrderConfig>,
) -> AirportConfig {
    AirportConfig {
        id,
        name: name.to_string(),
        location,
        runway_length_m: Some(3_000.0),
        fuel_price_per_l: Some(1.5),
        landing_fee_per_ton: Some(4.0),
        parking_fee_per_hour: Some(12.0),
        orders,
    }
}

#[test]
fn config_rejects_zero_restock_cycle() {
    let mut gameplay = base_gameplay();
    gameplay.restock_cycle_hours = 0;
    let cfg = WorldConfig {
        seed: Some(1),
        starting_cash: 400_000.0,
        airports: Vec::new(),
        num_airports: Some(2),
        gameplay,
        airplanes: None,
    };

    let err = Game::from_config(cfg).unwrap_err();
    match err {
        rusty_runways_core::utils::errors::GameError::InvalidConfig { msg } => {
            assert!(msg.contains("restock_cycle_hours"));
        }
        other => panic!("expected InvalidConfig error, got {:?}", other),
    }
}

#[test]
fn config_accepts_manual_passenger_orders() {
    let mut gameplay = base_gameplay();
    gameplay.orders.regenerate = false;
    gameplay.orders.generate_initial = false;
    let airports = vec![
        airport(
            0,
            "HUB",
            Some(Location { x: 800.0, y: 900.0 }),
            vec![ManualOrderConfig::Passengers {
                passengers: 18,
                value: 6_800.0,
                deadline_hours: 20,
                destination_id: 1,
            }],
        ),
        airport(
            1,
            "EDGE",
            None,
            vec![ManualOrderConfig::Cargo {
                cargo: CargoType::Electronics,
                weight: 420.0,
                value: 7_200.0,
                deadline_hours: 30,
                destination_id: 0,
            }],
        ),
    ];
    let cfg = WorldConfig {
        seed: Some(99),
        starting_cash: 600_000.0,
        airports,
        num_airports: None,
        gameplay,
        airplanes: None,
    };

    let game = Game::from_config(cfg).expect("config should build");
    assert_eq!(game.map.num_airports, 2);

    let first_orders = &game.map.airports[0].0.orders;
    assert!(
        first_orders
            .iter()
            .any(|o| matches!(o.payload, OrderPayload::Passengers { .. }))
    );
    let second_orders = &game.map.airports[1].0.orders;
    assert!(
        second_orders
            .iter()
            .any(|o| matches!(o.payload, OrderPayload::Cargo { .. }))
    );

    let generated_coord = game.map.airports[1].1;
    assert!((generated_coord.x, generated_coord.y) != (0.0, 0.0));
}

#[test]
fn world_event_global_branch_resets_prices() {
    let mut game = Game::new(23, Some(3), 150_000.0);
    let baseline: Vec<f32> = game
        .map
        .airports
        .iter()
        .map(|(ap, _)| ap.fuel_price)
        .collect();

    game.events.clear();
    game.events.push(ScheduledEvent {
        time: game.time,
        event: Event::WorldEvent {
            airport: None,
            factor: 1.5,
            duration: 6,
        },
    });

    assert!(game.tick_event());
    for ((airport, _), before) in game.map.airports.iter().zip(baseline.iter()) {
        assert!((airport.fuel_price - before * 1.5).abs() < 1e-3);
    }

    game.events.push(ScheduledEvent {
        time: game.time + 6,
        event: Event::WorldEventEnd {
            airport: None,
            factor: 1.5,
        },
    });
    game.time += 6;
    assert!(game.tick_event());
    for ((airport, _), before) in game.map.airports.iter().zip(baseline.iter()) {
        assert!((airport.fuel_price - before).abs() < 1e-3);
    }
}

#[test]
fn maintenance_cycle_marks_and_repairs_planes() {
    let mut game = Game::new(5, Some(2), 80_000.0);
    game.events.clear();
    game.airplanes[0].hours_since_maintenance = 2_000;
    game.airplanes[0].status = AirplaneStatus::Parked;

    game.events.push(ScheduledEvent {
        time: game.time,
        event: Event::MaintenanceCheck,
    });
    assert!(game.tick_event());
    assert!(game.airplanes[0].needs_maintenance);
    assert!(matches!(game.airplanes[0].status, AirplaneStatus::Broken));

    game.events.push(ScheduledEvent {
        time: game.time + 8,
        event: Event::Maintenance { plane: 0 },
    });
    game.advance(10);
    assert!(!game.airplanes[0].needs_maintenance);
    assert!(matches!(game.airplanes[0].status, AirplaneStatus::Parked));
}

#[test]
fn daily_stats_and_pricing_update() {
    let mut game = Game::new(7, Some(2), 90_000.0);
    let baseline = game.map.airports[0].0.fuel_price;
    game.events.clear();
    game.daily_income = 500.0;
    game.daily_expenses = 200.0;

    game.events.push(ScheduledEvent {
        time: game.time,
        event: Event::DailyStats,
    });
    game.events.push(ScheduledEvent {
        time: game.time,
        event: Event::DynamicPricing,
    });

    assert!(game.tick_event());
    assert!(game.tick_event());

    assert_eq!(game.stats.len(), 1);
    assert!(game.daily_income.abs() < f32::EPSILON);
    assert!(
        game.map
            .airports
            .iter()
            .any(|(ap, _)| (ap.fuel_price - baseline).abs() > 1e-3)
    );
}

#[test]
fn observe_and_listing_cover_passengers() {
    let mut game = Game::new(11, Some(2), 120_000.0);
    game.map.airports[0].0.orders.push(Order {
        id: 500,
        payload: OrderPayload::Passengers { count: 12 },
        value: 4_200.0,
        deadline: 18,
        origin_id: 0,
        destination_id: 1,
    });

    let obs = game.observe();
    assert_eq!(obs.planes.len(), game.airplanes.len());

    let _ = game.drain_log();
    game.reset_runtime();

    #[cfg(feature = "ui_prints")]
    {
        game.show_cash();
        game.show_time();
        game.show_stats();
        game.list_airports(true);
        assert!(game.list_airport(0, true).is_ok());
        assert!(game.list_airport(99, false).is_err());
    }
}

#[test]
fn save_and_load_roundtrip() {
    let tmp = tempdir().expect("tempdir");
    let original = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("set cwd");

    let mut game = Game::new(13, Some(2), 200_000.0);
    game.player.cash = 333_333.0;
    game.save_game("integration_roundtrip").expect("save");

    let loaded = Game::load_game("integration_roundtrip").expect("load");
    assert_eq!(loaded.player.cash, 333_333.0);

    std::env::set_current_dir(&original).expect("restore cwd");
}
