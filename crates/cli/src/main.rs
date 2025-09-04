use clap::Parser;
use rusty_runways_cli::commands::Command;
use rusty_runways_cli::read::{LineReaderHelper, print_banner};
use rusty_runways_cli::{
    cli::{Cli, init_game_from_cli},
    commands::parse_command,
};
use rusty_runways_core::Game;
use rusty_runways_core::utils::airplanes::models::AirplaneModel;
use rustyline::{ColorMode, CompletionType, Config, Editor};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    print_banner();
    let cli = Cli::parse();
    let mut game = match init_game_from_cli(cli) {
        Ok(game) => game,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // line parser
    let config = Config::builder()
        .completion_type(CompletionType::Circular)
        .color_mode(ColorMode::Enabled)
        .build();
    let mut line_reader = Editor::with_config(config)?;
    line_reader.set_helper(Some(LineReaderHelper::new()));

    loop {
        let line = line_reader.readline("> ")?;
        let _ = line_reader.add_history_entry(line.as_str());

        match parse_command(&line) {
            Ok(Command::ShowModels) => {
                // Print airplane models table
                println!(
                    "{:<16} {:>8} {:>8} {:>7} {:>8} {:>10} {:>12} {:>12}",
                    "Model", "Cruise", "Fuel", "Burn", "Oper/h", "Payload", "Price", "Runway"
                );
                println!(
                    "{:-<16} {:-<8} {:-<8} {:-<7} {:-<8} {:-<10} {:-<12} {:-<12}",
                    "", "", "", "", "", "", "", ""
                );
                let models = [
                    AirplaneModel::SparrowLight,
                    AirplaneModel::FalconJet,
                    AirplaneModel::CometRegional,
                    AirplaneModel::Atlas,
                    AirplaneModel::TitanHeavy,
                    AirplaneModel::Goliath,
                    AirplaneModel::Zephyr,
                    AirplaneModel::Lightning,
                ];
                for m in models {
                    let s = m.specs();
                    println!(
                        "{:<16} {:>8.0} {:>8.0} {:>7.0} {:>8.0} {:>10.0} {:>12.0} {:>12.0}",
                        format!("{:?}", m),
                        s.cruise_speed,
                        s.fuel_capacity,
                        s.fuel_consumption,
                        s.operating_cost,
                        s.payload_capacity,
                        s.purchase_price,
                        s.min_runway_length,
                    );
                }
            }
            Ok(Command::ShowAirports { with_orders }) => game.list_airports(with_orders),

            Ok(Command::ShowAirport { id, with_orders }) => {
                if let Err(e) = game.list_airport(id, with_orders) {
                    println!("{}", e);
                }
            }

            Ok(Command::ShowAirplanes) => {
                if let Err(e) = game.list_airplanes() {
                    println!("{}", e)
                }
            }

            Ok(Command::ShowAirplane { id }) => {
                if let Err(e) = game.list_airplane(id) {
                    println!("{}", e);
                }
            }

            Ok(Command::ShowDistances { plane_id }) => {
                if let Err(e) = game.show_distances(plane_id) {
                    println!("{}", e);
                }
            }

            Ok(Command::BuyPlane { model, airport }) => match game.buy_plane(&model, airport) {
                Ok(()) => {
                    println!("Airplane was bought!")
                }
                Err(e) => {
                    println!("{:?}", e)
                }
            },

            Ok(Command::LoadOrder { order, plane }) => {
                if let Err(e) = game.load_order(order, plane) {
                    println!("Load failed: {}", e);
                } else {
                    println!("Loading order {:?} onto plane {:?}", order, plane);
                }
            }

            Ok(Command::LoadOrders { orders, plane }) => {
                for o in orders {
                    if let Err(e) = game.load_order(o, plane) {
                        println!("Load failed: {}", e);
                    } else {
                        println!("Loading order {:?} onto plane {:?}", o, plane);
                    }
                }
            }

            Ok(Command::UnloadAll { plane }) => {
                if let Err(e) = game.unload_all(plane) {
                    println!("Unloading failed: {}", e)
                }
            }

            Ok(Command::UnloadOrder { order, plane }) => {
                if let Err(e) = game.unload_order(order, plane) {
                    println!("Unloading failed: {}", e)
                }
            }

            Ok(Command::UnloadOrders { orders, plane }) => {
                for o in orders {
                    if let Err(e) = game.unload_order(o, plane) {
                        println!("Unloading failed: {}", e);
                    }
                }
            }

            Ok(Command::Refuel { plane }) => {
                if let Err(e) = game.refuel_plane(plane) {
                    println!("Failed to refuel: {}", e);
                }
            }

            Ok(Command::DepartPlane { plane, dest }) => {
                if let Err(e) = game.depart_plane(plane, dest) {
                    println!("Cannot depart: {}", e);
                }
            }

            Ok(Command::ShowCash) => {
                game.show_cash();
            }

            Ok(Command::ShowTime) => {
                game.show_time();
            }

            Ok(Command::ShowStats) => {
                game.show_stats();
            }

            Ok(Command::Advance { hours }) => game.advance(hours),

            Ok(Command::Exit) => break,

            Ok(Command::SaveGame { name }) => {
                if let Err(e) = game.save_game(&name) {
                    println!("Failed to save: {}", e);
                } else {
                    println!("Successfully loaded game: {name}");
                }
            }

            Ok(Command::LoadGame { name }) => match Game::load_game(&name) {
                Ok(loaded_game) => {
                    game = loaded_game;
                }
                Err(e) => {
                    println!("Failed to load game: {}", e);
                }
            },

            Err(e) => println!("Syntax error: {}", e),
            _ => println!("Not yet implemented"),
        }
    }

    Ok(())
}
