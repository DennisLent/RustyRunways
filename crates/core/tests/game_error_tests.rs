use rusty_runways_core::Game;
use rusty_runways_core::utils::errors::GameError;

#[test]
fn depart_plane_invalid_id() {
    let mut game = Game::new(0, Some(1), 1_000.0);
    let err = game.depart_plane(99, 0).unwrap_err();
    assert!(matches!(err, GameError::PlaneIdInvalid { id: 99 }));
}
