use RustyRunways::commands::Command;
use RustyRunways::utils::read::{print_banner, read_line};
use RustyRunways::{commands::parse_command, game::Game};

fn main() {
    print_banner();
    let mut game = Game::new(1, Some(5), 1_000_000.0);

    loop {
        let line = read_line();
        match parse_command(&line) {
            Ok(Command::ShowAirports { with_orders }) => game.list_airports(with_orders),
            Ok(Command::ShowAirport { id, with_orders }) => {
                game.list_airport(id, with_orders);
            }
            Ok(Command::BuyPlane { model, airport }) => match game.buy_plane(&model, airport) {
                Ok(()) => {
                    println!("Airplane was bought!")
                }
                Err(e) => {
                    println!("{:?}", e)
                }
            },
            Ok(Command::LoadOrder { orders, plane }) => {
                for o in orders {
                    if let Err(e) = game.load_order(o, plane) {
                        println!("Load failed: {}", e);
                    }
                }
            }
            Ok(Command::DepartPlane { plane, dest }) => {
                if let Err(e) = game.depart_plane(plane, dest) {
                    println!("Cannot depart: {}", e);
                }
            }
            Ok(Command::Advance { hours }) => game.advance(hours),
            Ok(Command::Exit) => break,
            Err(e) => println!("Syntax error: {}", e),
            _ => println!("Not yet implemented"),
        }
    }
}
