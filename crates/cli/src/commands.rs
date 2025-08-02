#[derive(Debug)]
pub enum Command {
    ShowAirports { with_orders: bool },
    ShowAirport { id: usize, with_orders: bool },
    ShowAirplanes,
    ShowAirplane { id: usize },
    ShowDistances { plane_id: usize },
    BuyPlane { model: String, airport: usize },
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
    Exit,
    SaveGame { name: String },
    LoadGame { name: String },
}

fn parse_id_list(s: &str) -> Result<Vec<usize>, String> {
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
                .map_err(|_| format!("Invalid order id: `{}`", part))
        })
        .collect()
}

pub fn parse_command(line: &str) -> Result<Command, String> {
    let toks: Vec<&str> = line.split_whitespace().collect();

    if toks.len() >= 5 && toks[0] == "LOAD" && toks[1] == "ORDERS" {
        // find the "ON"
        if let Some(on_idx) = toks.iter().position(|&t| t == "ON") {
            // tokens [2..on_idx] are our ID list, re-join them:
            let orders_str = toks[2..on_idx].join(" ");
            let orders = parse_id_list(&orders_str)
                .map_err(|e| format!("Could not parse order list: {}", e))?;

            // next token must be the ids
            let plane = toks
                .get(on_idx + 1)
                .ok_or("Expected plane id after ON")?
                .parse()
                .map_err(|_| "bad plane id")?;
            return Ok(Command::LoadOrders { orders, plane });
        }
    }

    match toks.as_slice() {
        // Inspecting the world state
        ["SHOW", "AIRPORTS"] => Ok(Command::ShowAirports { with_orders: false }),

        ["SHOW", "AIRPORTS", "WITH", "ORDERS"] => Ok(Command::ShowAirports { with_orders: true }),

        ["SHOW", "AIRPORTS", id] => Ok(Command::ShowAirport {
            id: id.parse().map_err(|_| "bad airport id")?,
            with_orders: false,
        }),

        ["SHOW", "AIRPORTS", id, "WITH", "ORDERS"] => Ok(Command::ShowAirport {
            id: id.parse().map_err(|_| "bad airport id")?,
            with_orders: true,
        }),

        ["SHOW", "PLANES"] => Ok(Command::ShowAirplanes),

        ["SHOW", "PLANES", pid] => Ok(Command::ShowAirplane {
            id: pid.parse().map_err(|_| "bad plane id")?,
        }),

        ["SHOW", "DISTANCES", plane_id] => Ok(Command::ShowDistances {
            plane_id: plane_id.parse().map_err(|_| "bad plane id")?,
        }),

        // Purchases
        ["BUY", "PLANE", model, aid] => Ok(Command::BuyPlane {
            model: model.to_string(),
            airport: aid.parse().map_err(|_| "bad airport id")?,
        }),

        // Exit
        ["EXIT"] => Ok(Command::Exit),

        // Save and Load
        ["SAVE", name] => Ok(Command::SaveGame {
            name: name.to_string(),
        }),
        ["LOAD", name] => Ok(Command::LoadGame {
            name: name.to_string(),
        }),

        // Queries
        ["SHOW", "CASH"] => Ok(Command::ShowCash),
        ["SHOW", "TIME"] => Ok(Command::ShowTime),
        ["SHOW", "STATS"] => Ok(Command::ShowStats),

        // Time control
        ["ADVANCE", n] => Ok(Command::Advance {
            hours: n.parse().map_err(|_| "bad time n")?,
        }),

        [] => Ok(Command::Advance { hours: 1 }),

        // Dispatch & movement
        ["DEPART", "PLANE", plane_id, destination_airport_id] => Ok(Command::DepartPlane {
            plane: plane_id.parse().map_err(|_| "bad plane id")?,
            dest: destination_airport_id
                .parse()
                .map_err(|_| "bad airport id")?,
        }),

        ["HOLD", "PLANE", plane_id] => Ok(Command::HoldPlane {
            plane: plane_id.parse().map_err(|_| "bad plane id")?,
        }),

        // Cargo handling
        ["LOAD", "ORDER", order_id, "ON", plane_id] => Ok(Command::LoadOrder {
            order: order_id.parse().map_err(|_| "bad order id")?,
            plane: plane_id.parse().map_err(|_| "bad plane id")?,
        }),

        ["LOAD", "ORDERS", orders, "ON", plane_id] => {
            let order_vec = parse_id_list(orders)?;
            let plane = plane_id.parse::<usize>().map_err(|_| "bad plane id")?;

            Ok(Command::LoadOrders {
                orders: order_vec,
                plane,
            })
        }

        ["UNLOAD", "ORDER", order_id, "FROM", plane_id] => Ok(Command::UnloadOrder {
            order: order_id.parse().map_err(|_| "bad order id")?,
            plane: plane_id.parse().map_err(|_| "bad plane id")?,
        }),

        ["UNLOAD", "ORDERS", orders, "ON", plane_id] => {
            let order_vec = parse_id_list(orders)?;
            let plane = plane_id.parse::<usize>().map_err(|_| "bad plane id")?;

            Ok(Command::UnloadOrders {
                orders: order_vec,
                plane,
            })
        }

        ["UNLOAD", "ALL", "FROM", plane_id] => Ok(Command::UnloadAll {
            plane: plane_id.parse::<usize>().map_err(|_| "bad plane id")?,
        }),

        ["REFUEL", "PLANE", plane_id] => Ok(Command::Refuel {
            plane: plane_id.parse::<usize>().map_err(|_| "bad plane id")?,
        }),

        other => Err(format!("Unrecognized command: {:?}", other)),
    }
}
