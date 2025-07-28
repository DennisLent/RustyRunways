use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "commands.pest"]
pub struct DSLParser;

#[derive(Debug)]
pub enum Command {
    ShowAirports { with_orders: bool },
    ShowAirport { id: usize, with_orders: bool },
    ShowAirplanes,
    ShowAirplane { id: usize },
    BuyPlane { model: String, airport: usize },
    LoadOrder { orders: Vec<usize>, plane: usize },
    UnloadOrder { order: usize, plane: usize },
    UnloadAll { plane: usize },
    DepartPlane { plane: usize, dest: usize },
    HoldPlane { plane: usize },
    Advance { hours: u64 },
    ShowCash,
    ShowTime,
    Exit,
}

pub fn parse_command(input: &str) -> Result<Command, String> {

    let mut pairs = DSLParser::parse(Rule::script, input)
        .map_err(|e| format!("Parse error: {}", e))?;


    let command_pair = pairs
        .next()                      // first stmt
        .and_then(|stmt| stmt.into_inner().next()) // its command
        .ok_or_else(|| "No command found".to_string())?;


    match command_pair.as_rule() {
        Rule::show_cmd => {
            let mut inner = command_pair.into_inner();
            let kind = inner.next().unwrap();
            match kind.as_str() {
                k if k.starts_with("AIRPORTS") => {
                    let with_orders = k.contains("WITH");
                    Ok(Command::ShowAirports { with_orders })
                }
                "AIRPORT" => {
                    let id: usize = inner.next().unwrap().as_str().parse().unwrap();
                    let with_orders = inner
                        .next()
                        .map(|p| p.as_rule() == Rule::WITH_ORDERS)
                        .unwrap_or(false);
                    Ok(Command::ShowAirport { id, with_orders })
                }
                other if other.starts_with("AIRPLANES") => {
                    if let Some(idp) = inner.next() {
                        let id: usize = idp.as_str().parse().unwrap();
                        Ok(Command::ShowAirplane { id })
                    } else {
                        Ok(Command::ShowAirplanes)
                    }
                }
                _ => unreachable!(),
            }
        }
        Rule::buy_cmd => {
            let mut it = command_pair.into_inner();
            let model = it.next().unwrap().as_str().to_string();
            let airport: usize = it.next().unwrap().as_str().parse().unwrap();
            Ok(Command::BuyPlane { model, airport })
        }
        Rule::load_cmd => {
            let mut it = command_pair.into_inner();
            let orders = it
                .next()
                .unwrap()
                .into_inner()
                .map(|p| p.as_str().parse().unwrap())
                .collect();
            let plane: usize = it.next().unwrap().as_str().parse().unwrap();
            Ok(Command::LoadOrder { orders, plane })
        }
        Rule::advance_cmd => {
            let hours: u64 = command_pair.into_inner().next().unwrap().as_str().parse().unwrap();
            Ok(Command::Advance { hours })
        }
        Rule::exit_cmd => Ok(Command::Exit),
        Rule::query_cmd => {
            let inner = command_pair.into_inner().next().unwrap().as_str();
            match inner {
                "CASH" => Ok(Command::ShowCash),
                "TIME" => Ok(Command::ShowTime),
                _ => unreachable!(),
            }
        }
        
        _ => Err("Unknown command".into()),
    }
}

