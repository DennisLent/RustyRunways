use clap::Parser;
use rand::Rng;

use rusty_runways_core::Game;

/// Command line arguments for configuring the game.
#[derive(Parser, Debug)]
pub struct Cli {
    /// Seed used for deterministic world generation
    #[arg(long)]
    pub seed: Option<u64>,
    /// Number of airports in the generated world
    #[arg(long)]
    pub n: Option<usize>,
    /// Starting cash for the player
    #[arg(long, default_value_t = 1_000_000.0)]
    pub c: f32,
}

/// Initialize a [`Game`] from command line arguments.
///
/// * If both `seed` and `n` are provided, they are used verbatim.
/// * If neither are provided, random values are generated.
/// * Supplying only one of `seed` or `n` results in an error.
pub fn init_game_from_cli(cli: Cli) -> Result<Game, String> {
    match (cli.seed, cli.n) {
        (Some(seed), Some(n)) => Ok(Game::new(seed, Some(n), cli.c)),
        (None, None) => {
            let seed = rand::thread_rng().r#gen();
            Ok(Game::new(seed, None, cli.c))
        }
        _ => Err("Both --seed and --n must be specified".to_string()),
    }
}
