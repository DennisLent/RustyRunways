use RustyRunways::game::Game;
#[test]
fn test_game_new() {
    let game = Game::new(1, Some(2), 100.0);
    assert_eq!(game.map.num_airports, 2);
}
