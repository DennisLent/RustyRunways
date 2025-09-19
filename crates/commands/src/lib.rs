use std::fmt;

#[derive(Debug)]
pub enum Command {
    ShowAirports { with_orders: bool },
    ShowAirport { id: usize, with_orders: bool },
    ShowAirplanes,
    ShowAirplane { id: usize },
    ShowDistances { plane_id: usize },
    BuyPlane { model: String, airport: usize },
    SellPlane { plane: usize },
    LoadOrder { order: usize, plane: usize },
    LoadOrders { orders: Vec<usize>, plane: usize },
    UnloadOrder { order: usize, plane: usize },
    UnloadOrders { orders: Vec<usize>, plane: usize },
    UnloadAll { plane: usize },
    Refuel { plane: usize },
    DepartPlane { plane: usize, dest: usize },
    HoldPlane { plane: usize },
    Advance { hours: u64 },
    ShowCash,
    ShowTime,
    ShowStats,
    ShowModels,
    LoadConfig { path: String },
    Exit,
    SaveGame { name: String },
    LoadGame { name: String },
    Maintenance { plane_id: usize },
}

#[derive(Debug)]
pub enum CommandError {
    Syntax(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Syntax(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

type Result<T> = std::result::Result<T, CommandError>;

fn parse_id_list(s: &str) -> Result<Vec<usize>> {
    let inner = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len() - 1]
    } else {
        s
    };

    inner
        .split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            part.trim()
                .parse::<usize>()
                .map_err(|_| CommandError::Syntax(format!("Invalid order id: `{}`", part)))
        })
        .collect()
}

pub fn parse_command(line: &str) -> Result<Command> {
    let toks: Vec<&str> = line.split_whitespace().collect();

    if toks.len() >= 5 && toks[0] == "LOAD" && toks[1] == "ORDERS" {
        if let Some(on_idx) = toks.iter().position(|&t| t == "ON") {
            let orders_str = toks[2..on_idx].join(" ");
            let orders = parse_id_list(&orders_str)
                .map_err(|e| CommandError::Syntax(format!("Could not parse order list: {}", e)))?;

            let plane = toks
                .get(on_idx + 1)
                .ok_or_else(|| CommandError::Syntax("Expected plane id after ON".into()))?
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?;
            return Ok(Command::LoadOrders { orders, plane });
        }
    }

    match toks.as_slice() {
        ["SHOW", "AIRPORTS"] => Ok(Command::ShowAirports { with_orders: false }),
        ["SHOW", "AIRPORTS", "WITH", "ORDERS"] => Ok(Command::ShowAirports { with_orders: true }),
        ["SHOW", "AIRPORTS", id] => Ok(Command::ShowAirport {
            id: id
                .parse()
                .map_err(|_| CommandError::Syntax("bad airport id".into()))?,
            with_orders: false,
        }),
        ["SHOW", "AIRPORTS", id, "WITH", "ORDERS"] => Ok(Command::ShowAirport {
            id: id
                .parse()
                .map_err(|_| CommandError::Syntax("bad airport id".into()))?,
            with_orders: true,
        }),
        ["SHOW", "PLANES"] => Ok(Command::ShowAirplanes),
        ["SHOW", "PLANES", pid] => Ok(Command::ShowAirplane {
            id: pid
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["SHOW", "DISTANCES", plane_id] => Ok(Command::ShowDistances {
            plane_id: plane_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["BUY", "PLANE", model, aid] => Ok(Command::BuyPlane {
            model: model.to_string(),
            airport: aid
                .parse()
                .map_err(|_| CommandError::Syntax("bad airport id".into()))?,
        }),
        ["SELL", "PLANE", plane_id] => Ok(Command::SellPlane {
            plane: plane_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["EXIT"] => Ok(Command::Exit),
        ["SAVE", name] => Ok(Command::SaveGame {
            name: name.to_string(),
        }),
        ["LOAD", name] => Ok(Command::LoadGame {
            name: name.to_string(),
        }),
        ["SHOW", "CASH"] => Ok(Command::ShowCash),
        ["SHOW", "TIME"] => Ok(Command::ShowTime),
        ["SHOW", "STATS"] => Ok(Command::ShowStats),
        ["SHOW", "MODELS"] => Ok(Command::ShowModels),
        ["ADVANCE", n] => Ok(Command::Advance {
            hours: n
                .parse()
                .map_err(|_| CommandError::Syntax("bad time n".into()))?,
        }),
        ["LOAD", "CONFIG", path] => Ok(Command::LoadConfig {
            path: path.to_string(),
        }),
        [] => Ok(Command::Advance { hours: 1 }),
        ["DEPART", "PLANE", plane_id, destination_airport_id] => Ok(Command::DepartPlane {
            plane: plane_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
            dest: destination_airport_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad airport id".into()))?,
        }),
        ["HOLD", "PLANE", plane_id] => Ok(Command::HoldPlane {
            plane: plane_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["MAINTENANCE", plane_id] => Ok(Command::Maintenance {
            plane_id: plane_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["LOAD", "ORDER", order_id, "ON", plane_id] => Ok(Command::LoadOrder {
            order: order_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad order id".into()))?,
            plane: plane_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["LOAD", "ORDERS", orders, "ON", plane_id] => {
            let order_vec = parse_id_list(orders)?;
            let plane = plane_id
                .parse::<usize>()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?;
            Ok(Command::LoadOrders {
                orders: order_vec,
                plane,
            })
        }
        ["UNLOAD", "ORDER", order_id, "FROM", plane_id] => Ok(Command::UnloadOrder {
            order: order_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad order id".into()))?,
            plane: plane_id
                .parse()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["UNLOAD", "ORDERS", orders, "ON", plane_id] => {
            let order_vec = parse_id_list(orders)?;
            let plane = plane_id
                .parse::<usize>()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?;
            Ok(Command::UnloadOrders {
                orders: order_vec,
                plane,
            })
        }
        ["UNLOAD", "ALL", "FROM", plane_id] => Ok(Command::UnloadAll {
            plane: plane_id
                .parse::<usize>()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        ["REFUEL", "PLANE", plane_id] => Ok(Command::Refuel {
            plane: plane_id
                .parse::<usize>()
                .map_err(|_| CommandError::Syntax("bad plane id".into()))?,
        }),
        other => Err(CommandError::Syntax(format!(
            "Unrecognized command: {:?}",
            other
        ))),
    }
}
