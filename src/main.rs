mod utils;
mod player;
mod game;
mod events;

use crate::game::Game;


fn main() {
    let game = Game::new(1, Some(4), 1_000_000.0);
    println!("game map: {:?}", game.map);
    println!("game time: {:?}", game.time);

}
