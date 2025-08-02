use clap::Parser;
use rusty_runways_cli::cli::{Cli, init_game_from_cli};

#[test]
fn cli_requires_seed_and_n() {
    let cli = Cli::try_parse_from(["test", "--seed", "1"]).unwrap();
    assert!(init_game_from_cli(cli).is_err());
}

#[test]
fn cli_defaults_cash() {
    let cli = Cli::try_parse_from(["test", "--seed", "1", "--n", "5"]).unwrap();
    let game = init_game_from_cli(cli).unwrap();
    assert_eq!(game.map.seed, 1);
    assert_eq!(game.map.num_airports, 5);
    assert_eq!(game.player.cash, 1_000_000.0);
}

#[test]
fn cli_accepts_custom_cash() {
    let cli = Cli::try_parse_from(["test", "--seed", "2", "--n", "6", "--c", "2000000"]).unwrap();
    let game = init_game_from_cli(cli).unwrap();
    assert_eq!(game.map.seed, 2);
    assert_eq!(game.map.num_airports, 6);
    assert_eq!(game.player.cash, 2_000_000.0);
}

#[test]
fn cli_random_when_no_args() {
    let cli = Cli::try_parse_from(["test"]).unwrap();
    let game = init_game_from_cli(cli).unwrap();
    assert!(game.map.num_airports >= 4 && game.map.num_airports <= 10);
    assert_eq!(game.player.cash, 1_000_000.0);
}
