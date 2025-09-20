use rusty_runways_core::Game;
use rusty_runways_core::utils::errors::GameError;

fn new_small_game() -> Game {
    Game::new(1, Some(5), 650_000.0)
}

#[test]
fn depart_runway_too_short_errors() {
    let mut game = new_small_game();
    // choose plane 0 and a destination different from its current airport id
    let plane_id = 0usize;
    let current_loc = game.airplanes[plane_id].location;
    // pick a destination within range so runway check is hit before range check
    let max_range = game.airplanes[plane_id].max_range();
    let dest_id = game
        .map
        .airports
        .iter()
        .filter(|(_, c)| *c != current_loc)
        .filter(|(_, c)| {
            let dx = current_loc.x - c.x;
            let dy = current_loc.y - c.y;
            let dist = (dx * dx + dy * dy).sqrt();
            dist <= max_range
        })
        .map(|(a, _)| a.id)
        .next()
        .expect("expected at least one in-range destination");

    // force destination runway too short
    if let Some((ap, _)) = game.map.airports.iter_mut().find(|(a, _)| a.id == dest_id) {
        ap.runway_length = 10.0;
    }

    let err = game.depart_plane(plane_id, dest_id).unwrap_err();
    assert!(matches!(err, GameError::RunwayTooShort { .. }));
}

#[test]
fn depart_out_of_range_errors() {
    let mut game = new_small_game();
    let plane_id = 0usize;
    // drain fuel to force out-of-range
    game.airplanes[plane_id].current_fuel = 1.0; // 1L

    let current_loc = game.airplanes[plane_id].location;
    let dest_id = game
        .map
        .airports
        .iter()
        .find(|(_, c)| *c != current_loc)
        .map(|(a, _)| a.id)
        .unwrap();

    let err = game.depart_plane(plane_id, dest_id).unwrap_err();
    assert!(matches!(err, GameError::OutOfRange { .. }));
}

#[test]
fn refuel_invalid_plane_id() {
    let mut game = new_small_game();
    let err = game.refuel_plane(999).unwrap_err();
    assert!(matches!(err, GameError::PlaneIdInvalid { .. }));
}

#[test]
fn load_order_invalid_id() {
    let mut game = new_small_game();
    let plane_id = 0usize;
    let err = game.load_order(999_999, plane_id).unwrap_err();
    assert!(matches!(err, GameError::OrderIdInvalid { .. }));
}

#[test]
fn unload_order_invalid_id() {
    let mut game = new_small_game();
    let plane_id = 0usize;
    let err = game.unload_order(12345, plane_id).unwrap_err();
    assert!(matches!(err, GameError::OrderIdInvalid { .. }));
}
