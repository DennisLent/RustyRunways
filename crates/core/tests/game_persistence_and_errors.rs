use rusty_runways_core::game::Game;
use rusty_runways_core::utils::{
    airplanes::models::AirplaneStatus, coordinate::Coordinate, errors::GameError,
};

#[test]
fn save_and_load_roundtrip_and_execute_mapping() {
    let mut game = Game::new(42, Some(5), 123_456.0);

    // Execute a couple of commands (parsing and dispatch only)
    game.execute_str("SHOW TIME").unwrap();
    game.execute_str("SHOW AIRPORTS").unwrap();
    game.execute_str("ADVANCE 3").unwrap();

    // Save then load
    let name = "cov_test_save";
    game.save_game(name).unwrap();
    let loaded = Game::load_game(name).unwrap();
    assert_eq!(loaded.seed(), 42);

    // Reset runtime fields and drain log for coverage
    let mut g2 = loaded;
    g2.reset_runtime();
    let _ = g2.drain_log();
}

#[test]
fn list_airport_invalid_and_find_associated_airport_error() {
    #[cfg(feature = "ui_prints")]
    {
        let mut game = Game::new(1, Some(5), 650_000.0);

        // invalid airport id
        let err = game.list_airport(9999, true).unwrap_err();
        assert!(matches!(err, GameError::AirportIdInvalid { .. }));

        // Force plane to be parked at a location which isn't any airport
        let plane = &mut game.airplanes[0];
        plane.status = AirplaneStatus::Parked;
        plane.location = Coordinate::new(0.123, 0.456);
        let err = game.list_airplane(0).unwrap_err();
        assert!(matches!(err, GameError::AirportLocationInvalid { .. }));
    }
}
