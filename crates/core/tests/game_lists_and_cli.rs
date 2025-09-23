#![cfg(feature = "ui_prints")]

use rusty_runways_core::game::Game;
use rusty_runways_core::utils::airplanes::models::AirplaneStatus;

#[test]
fn lists_and_cli_commands_cover_print_paths() {
    // Deterministic seed
    let mut game = Game::new(1, Some(6), 650_000.0);

    // airport listings with and without orders
    game.list_airports(false);
    game.list_airport(0, true).unwrap();

    // airplane listings (parked)
    game.list_airplanes().unwrap();
    game.list_airplane(0).unwrap();

    // distances when parked (prints distances and landability)
    game.show_distances(0).unwrap();

    // advance to hit DynamicPricing and DailyStats at least once
    game.advance(24);

    // Show helpers (printing only)
    game.show_cash();
    game.show_time();
    game.show_stats();

    // depart to make plane in transit then show distances again (in‑transit branch)
    // choose a reachable destination deterministically: first airport not at origin within range
    let origin = game.planes()[0].location;
    let dest_id = game
        .airports()
        .iter()
        .enumerate()
        .find(|(_, (_, c))| *c != origin)
        .map(|(i, _)| i)
        .expect("need at least two airports");
    // try to depart; if out of range due to random spacing, just advance and skip
    if game.depart_plane(0, dest_id).is_ok() {
        assert!(matches!(
            game.planes()[0].status,
            AirplaneStatus::InTransit { .. }
        ));
        game.show_distances(0).unwrap();
    }
}
