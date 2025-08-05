use rusty_runways_core::{Game, utils::airplanes::models::AirplaneStatus};

#[test]
fn maintenance_event_schedules_and_resolves() {
    let mut game = Game::new(1, Some(2), 1_000.0);
    game.maintenance_on_airplane(0)
        .expect("maintenance should start");
    assert!(matches!(
        game.planes()[0].status,
        AirplaneStatus::Maintenance
    ));
    game.advance(1);
    assert!(matches!(game.planes()[0].status, AirplaneStatus::Parked));
}
