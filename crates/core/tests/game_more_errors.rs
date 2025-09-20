use rusty_runways_core::Game;
use rusty_runways_core::utils::errors::GameError;

fn new_small_game() -> Game {
    Game::new(1, Some(5), 650_000.0)
}

#[test]
fn depart_to_same_airport_errors() {
    let mut game = new_small_game();
    let plane_id = 0usize;
    // plane's current airport id is the one whose coord matches its location
    let current_airport_id = game
        .airports()
        .iter()
        .find(|(_, c)| *c == game.planes()[plane_id].location)
        .map(|(a, _)| a.id)
        .expect("plane should be at an airport at t0");

    let err = game.depart_plane(plane_id, current_airport_id).unwrap_err();
    assert!(matches!(err, GameError::SameAirport));
}

#[test]
fn load_order_while_in_transit_is_plane_not_at_airport() {
    let mut game = new_small_game();
    let plane_id = 0usize;

    // choose an in-range destination different from origin so we enter InTransit status
    let origin = game.planes()[plane_id].location;
    let max_range = game.planes()[plane_id].max_range();
    let dest_id = game
        .airports()
        .iter()
        .filter(|(_, c)| *c != origin)
        .filter(|(_, c)| {
            let dx = origin.x - c.x;
            let dy = origin.y - c.y;
            let dist = (dx * dx + dy * dy).sqrt();
            dist <= max_range
        })
        .map(|(a, _)| a.id)
        .next()
        .expect("expected at least one in-range destination");

    game.depart_plane(plane_id, dest_id).unwrap();
    // progress flight so location is not at an airport anymore
    game.advance(1);

    // Try to load any order (id doesn't matter) while the plane is mid‑air
    let err = game.load_order(123456, plane_id).unwrap_err();
    assert!(matches!(err, GameError::PlaneNotAtAirport { .. }));
}
