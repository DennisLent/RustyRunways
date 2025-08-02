use rusty_runways_core::{game::Game, utils::airplanes::models::AirplaneStatus};

#[test]
fn airplane_location_updates_during_flight() {
    let mut game = Game::new(1, Some(5), 10_000_000.0);
    let plane_id = 0;

    // determine origin index
    let origin_idx = game
        .map
        .airports
        .iter()
        .position(|(_, coord)| *coord == game.airplanes[plane_id].location)
        .unwrap();
    let origin_coord = game.airplanes[plane_id].location;

    // find a destination that requires at least 2 hours of flight
    let (dest_idx, dest_coord) = game
        .map
        .airports
        .iter()
        .enumerate()
        .find(|(idx, (airport, coord))| {
            *idx != origin_idx && game.airplanes[plane_id].can_fly_to(airport, coord).is_ok() && {
                let dist = game.airplanes[plane_id].distance_to(coord);
                let hours = (dist / game.airplanes[plane_id].specs.cruise_speed).ceil() as u64;
                hours >= 2
            }
        })
        .map(|(idx, (_a, c))| (idx, *c))
        .expect("No suitable destination found");

    game.depart_plane(plane_id, dest_idx).unwrap();

    let total_hours = if let AirplaneStatus::InTransit {
        hours_remaining, ..
    } = game.airplanes[plane_id].status
    {
        hours_remaining
    } else {
        panic!("Plane not in transit");
    };

    for hour in 1..=total_hours {
        game.advance(1);
        let loc = game.airplanes[plane_id].location;
        let fraction = hour as f32 / total_hours as f32;
        let expected_x = origin_coord.x + (dest_coord.x - origin_coord.x) * fraction;
        let expected_y = origin_coord.y + (dest_coord.y - origin_coord.y) * fraction;
        assert!((loc.x - expected_x).abs() < 1e-3);
        assert!((loc.y - expected_y).abs() < 1e-3);
    }

    assert!(matches!(
        game.airplanes[plane_id].status,
        AirplaneStatus::Parked
    ));
    let final_loc = game.airplanes[plane_id].location;
    assert!((final_loc.x - dest_coord.x).abs() < 1e-3);
    assert!((final_loc.y - dest_coord.y).abs() < 1e-3);
}
