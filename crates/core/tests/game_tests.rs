use rusty_runways_core::{
    Game,
    events::{Event, ScheduledEvent},
    utils::{
        airplanes::models::AirplaneStatus,
        errors::GameError,
        orders::{
            cargo::CargoType,
            order::{Order, OrderPayload},
        },
    },
};

#[test]
fn advance_zero_hours_keeps_time_and_events() {
    let mut game = Game::new(1, Some(4), 650_000.0);
    let before_time = game.time;
    let before_len = game.events.len();
    game.advance(0);
    assert_eq!(game.time, before_time);
    assert_eq!(game.events.len(), before_len);
}

#[test]
fn tick_event_returns_false_when_empty() {
    let mut game = Game::new(1, Some(3), 650_000.0);
    game.events.clear();
    assert!(!game.tick_event());
}

#[test]
fn initial_events_are_scheduled() {
    let game = Game::new(1, Some(3), 650_000.0);
    let mut has_restock = false;
    let mut has_daily = false;
    let mut has_dynamic = false;
    let mut has_world = false;
    let mut has_maint = false;
    for scheduled in game.events.clone().into_sorted_vec() {
        match scheduled.event {
            Event::Restock => has_restock = true,
            Event::DailyStats => has_daily = true,
            Event::DynamicPricing => has_dynamic = true,
            Event::WorldEvent { .. } => has_world = true,
            Event::MaintenanceCheck => has_maint = true,
            _ => {}
        }
    }
    assert!(has_restock && has_daily && has_dynamic && has_world && has_maint);
}

#[test]
fn advance_runs_events_up_to_target() {
    let mut game = Game::new(1, Some(3), 650_000.0);
    game.events.clear();
    game.airplanes[0].status = AirplaneStatus::Loading;
    game.events.push(ScheduledEvent {
        time: 1,
        event: Event::LoadingEvent { plane: 0 },
    });
    game.events.push(ScheduledEvent {
        time: 5,
        event: Event::LoadingEvent { plane: 0 },
    });
    game.advance(2);
    assert_eq!(game.time, 2);
    assert!(matches!(game.airplanes[0].status, AirplaneStatus::Parked));
    assert_eq!(game.events.peek().unwrap().time, 5);
}

#[test]
fn advance_executes_event_at_target_time() {
    let mut game = Game::new(1, Some(3), 650_000.0);
    game.events.clear();
    game.airplanes[0].status = AirplaneStatus::Loading;
    game.events.push(ScheduledEvent {
        time: 4,
        event: Event::LoadingEvent { plane: 0 },
    });
    game.advance(4);
    assert_eq!(game.time, 4);
    assert!(matches!(game.airplanes[0].status, AirplaneStatus::Parked));
    assert!(game.events.is_empty());
}

#[test]
fn tick_event_returns_true_when_event_present() {
    let mut game = Game::new(1, Some(2), 650_000.0);
    let res = game.tick_event();
    assert!(res);
}

#[test]
fn advance_zero_executes_due_events() {
    let mut game = Game::new(1, Some(2), 650_000.0);
    game.events.clear();
    game.airplanes[0].status = AirplaneStatus::Loading;
    game.events.push(ScheduledEvent {
        time: 0,
        event: Event::LoadingEvent { plane: 0 },
    });
    game.advance(0);
    assert_eq!(game.time, 0);
    assert!(matches!(game.airplanes[0].status, AirplaneStatus::Parked));
    assert!(game.events.is_empty());
}

#[test]
fn advance_processes_all_events_at_same_time() {
    let mut game = Game::new(1, Some(2), 650_000.0);
    game.events.clear();
    game.airplanes[0].status = AirplaneStatus::Loading;
    game.events.push(ScheduledEvent {
        time: 3,
        event: Event::LoadingEvent { plane: 0 },
    });
    game.events.push(ScheduledEvent {
        time: 3,
        event: Event::MaintenanceCheck,
    });
    game.advance(3);
    assert_eq!(game.time, 3);
    assert!(game.events.peek().is_none_or(|e| e.time > 3));
}

#[test]
fn sell_plane_requires_plane_to_be_parked() {
    let mut game = Game::new(2, Some(3), 650_000.0);
    let origin = game.airplanes[0].location;
    let destination = game.map.airports[1].0.id;
    game.airplanes[0].status = AirplaneStatus::InTransit {
        hours_remaining: 4,
        destination,
        origin,
        total_hours: 4,
    };
    let err = game.sell_plane(0).unwrap_err();
    assert!(matches!(err, GameError::PlaneNotReady { .. }));
}

#[test]
fn sell_plane_requires_empty_manifest() {
    let mut game = Game::new(3, Some(3), 650_000.0);
    game.airplanes[0].status = AirplaneStatus::Parked;
    game.airplanes[0].manifest.push(Order {
        id: 42,
        payload: OrderPayload::Cargo {
            cargo_type: CargoType::Food,
            weight: 10.0,
        },
        value: 500.0,
        deadline: 12,
        origin_id: 0,
        destination_id: 1,
    });
    let err = game.sell_plane(0).unwrap_err();
    assert!(matches!(err, GameError::InvalidCommand { .. }));
}

#[test]
fn observe_reports_passenger_payload() {
    let game = Game::new(6, Some(4), 750_000.0);
    let snapshot = game.observe();
    assert!(!snapshot.planes.is_empty());
    let payload = &snapshot.planes[0].payload;
    assert_eq!(payload.passenger_current, 0);
    assert!(payload.passenger_capacity >= payload.passenger_current);
}

#[test]
fn world_event_cycle_updates_prices() {
    let mut game = Game::new(7, Some(4), 500_000.0);
    let base_price = game.map.airports[0].0.fuel_price;
    game.events.push(ScheduledEvent {
        time: game.time,
        event: Event::WorldEvent {
            airport: Some(0),
            factor: 1.2,
            duration: 2,
        },
    });
    assert!(game.tick_event());
    let increased = game.map.airports[0].0.fuel_price;
    assert!((increased - base_price * 1.2).abs() < 1e-3);

    game.events.clear();
    game.events.push(ScheduledEvent {
        time: game.time + 2,
        event: Event::WorldEventEnd {
            airport: Some(0),
            factor: 1.2,
        },
    });
    game.time += 2;
    assert!(game.tick_event());
    game.events.clear();
    let reset = game.map.airports[0].0.fuel_price;
    assert!((reset - base_price).abs() < 1e-3);
}

#[test]
fn sell_plane_updates_cash_and_daily_income() {
    let mut game = Game::new(4, Some(3), 650_000.0);
    let price = game.airplanes[0].specs.purchase_price;
    let refund = game.sell_plane(0).expect("sale should succeed");
    assert!((refund - price * 0.6).abs() < f32::EPSILON);
    assert!(game.airplanes.iter().all(|plane| plane.id != 0));
    assert!(game.player.fleet.iter().all(|plane| plane.id != 0));
    assert!(!game.arrival_times.contains_key(&0));
    assert!((game.player.cash - (650_000.0 + refund)).abs() < 1e-3);
    assert!((game.daily_income - refund).abs() < f32::EPSILON);
    assert_eq!(game.player.fleet_size, game.player.fleet.len());
}

#[test]
fn refuel_plane_requires_sufficient_cash() {
    let mut game = Game::new(5, Some(3), 650_000.0);
    game.player.cash = 1.0;
    game.airplanes[0].current_fuel = 0.0;
    let err = game.refuel_plane(0).unwrap_err();
    assert!(matches!(err, GameError::InsufficientFunds { .. }));
    assert!(game.player.cash <= 1.0);
}
