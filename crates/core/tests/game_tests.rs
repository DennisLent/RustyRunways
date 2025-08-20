use rusty_runways_core::{
    Game,
    events::{Event, ScheduledEvent},
    utils::airplanes::models::AirplaneStatus,
};

#[test]
fn advance_zero_hours_keeps_time_and_events() {
    let mut game = Game::new(1, Some(4), 1_000_000.0);
    let before_time = game.time;
    let before_len = game.events.len();
    game.advance(0);
    assert_eq!(game.time, before_time);
    assert_eq!(game.events.len(), before_len);
}

#[test]
fn tick_event_returns_false_when_empty() {
    let mut game = Game::new(1, Some(3), 1_000_000.0);
    game.events.clear();
    assert!(!game.tick_event());
}

#[test]
fn initial_events_are_scheduled() {
    let game = Game::new(1, Some(3), 1_000_000.0);
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
    let mut game = Game::new(1, Some(3), 1_000_000.0);
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
    let mut game = Game::new(1, Some(3), 1_000_000.0);
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
    let mut game = Game::new(1, Some(2), 1_000_000.0);
    let res = game.tick_event();
    assert!(res);
}

#[test]
fn advance_zero_executes_due_events() {
    let mut game = Game::new(1, Some(2), 1_000_000.0);
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
    let mut game = Game::new(1, Some(2), 1_000_000.0);
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
    assert!(game.events.peek().map_or(true, |e| e.time > 3));
}
