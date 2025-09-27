use rusty_runways_core::game::Game;
use rusty_runways_core::utils::airplanes::models::AirplaneModel;

#[test]
fn buy_plane_increases_daily_expenses() {
    let mut game = Game::new(1, Some(5), 100_000_000.0);

    let before = game.daily_expenses;
    let price = AirplaneModel::FalconJet.specs().purchase_price;

    game.buy_plane(&"FalconJet".to_string(), 0).unwrap();

    assert!((game.daily_expenses - (before + price)).abs() < 1e-3);
}
