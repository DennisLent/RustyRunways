use rusty_runways_core::game::Game;
use rusty_runways_core::utils::airplanes::models::AirplaneStatus;

#[test]
fn deliver_and_store_paths_and_world_events() {
    let mut game = Game::new(2, Some(8), 5_000_000.0);

    // Find a loadable order at origin that is reachable
    let plane_id = 0usize;
    let origin_loc = game.planes()[plane_id].location;
    let origin_idx = game
        .airports()
        .iter()
        .position(|(_, c)| *c == origin_loc)
        .unwrap();

    // pick first order the plane can carry and reach
    let mut chosen: Option<(usize, usize)> = None;
    for o in &game.map.airports[origin_idx].0.orders {
        let Some(weight) = o.cargo_weight() else {
            continue;
        };
        let (a, c) = &game.airports()[o.destination_id];
        if game.planes()[plane_id].can_fly_to(a, c).is_ok()
            && (game.planes()[plane_id].current_payload + weight)
                <= game.planes()[plane_id].specs.payload_capacity
        {
            chosen = Some((o.id, o.destination_id));
            break;
        }
    }

    if let Some((order_id, dest_id)) = chosen {
        // load then depart to destination
        game.load_order(order_id, plane_id).unwrap();
        game.advance(1);
        game.depart_plane(plane_id, dest_id).unwrap();

        // flight progress: advance until arrival
        let hours = match game.planes()[plane_id].status {
            AirplaneStatus::InTransit {
                hours_remaining, ..
            } => hours_remaining,
            _ => 0,
        };
        if hours > 0 {
            game.advance(hours);
        }
        // Now deliver
        game.unload_all(plane_id).unwrap();
    }

    // World event coverage: advance far enough to trigger at least one event
    game.advance(150);
}
