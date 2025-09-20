use clap::Parser;
use rand::Rng;

use rusty_runways_core::Game;

/// Command line arguments for configuring the game.
#[derive(Parser, Debug)]
pub struct Cli {
    /// Load world from YAML config file
    #[arg(long)]
    pub config: Option<String>,
    /// Seed used for deterministic world generation
    #[arg(long)]
    pub seed: Option<u64>,
    /// Number of airports in the generated world
    #[arg(long)]
    pub n: Option<usize>,
    /// Starting cash for the player
    #[arg(long, default_value_t = 650_000.0)]
    pub c: f32,
}

/// Initialize a [`Game`] from command line arguments.
///
/// * If both `seed` and `n` are provided, they are used verbatim.
/// * If neither are provided, random values are generated.
/// * Supplying only one of `seed` or `n` results in an error.
pub fn init_game_from_cli(cli: Cli) -> Result<Game, String> {
    if let Some(path) = cli.config {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read config {}: {}", path, e))?;
        let cfg: rusty_runways_core::config::WorldConfig =
            serde_yaml::from_str(&text).map_err(|e| format!("invalid yaml: {}", e))?;
        return rusty_runways_core::Game::from_config(cfg).map_err(|e| e.to_string());
    }
    match (cli.seed, cli.n) {
        (Some(seed), Some(n)) => Ok(Game::new(seed, Some(n), cli.c)),
        (None, None) => {
            let seed = rand::thread_rng().r#gen();
            Ok(Game::new(seed, None, cli.c))
        }
        _ => Err("Both --seed and --n must be specified".to_string()),
    }
}
