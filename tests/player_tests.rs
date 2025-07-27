use RustyRunways::player::Player;
use RustyRunways::utils::map::Map;

#[test]
fn player_initialization_gives_single_plane() {
    let map = Map::generate_from_seed(7, Some(4));
    let player = Player::new(1_000_000.0, &map);
    assert_eq!(player.fleet_size, 1);
    assert_eq!(player.fleet.len(), 1);
    assert_eq!(player.orders_delivered, 0);

    let (_min_distance, start_index) = map.min_distance();
    let expected = map.airports[start_index].1;
    let plane = &player.fleet[0];
    assert_eq!(plane.location.x, expected.x);
    assert_eq!(plane.location.y, expected.y);

    // chosen model must have range over min_distance
    let model_range = plane.max_range();
    let (distance, _) = map.min_distance();
    assert!(model_range > distance);
}
