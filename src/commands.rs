#[derive(Debug)]
pub enum Command {
    ShowAirports { with_orders: bool },
    ShowAirport { id: usize, with_orders: bool },
    ShowAirplanes,
    ShowAirplane { id: usize },
    BuyPlane { model: String, airport: usize },
    LoadOrder { order: usize, plane: usize },
    LoadOrders { orders: Vec<usize>, plane: usize },
    UnloadOrder { order: usize, plane: usize },
    UnloadAll { plane: usize },
    DepartPlane { plane: usize, dest: usize },
    HoldPlane { plane: usize },
    Advance { hours: u64 },
    ShowCash,
    ShowTime,
    Exit,
}

fn parse_id_list(s: &str) -> Result<Vec<usize>, String> {
    
    let inner = if s.starts_with('[') && s.ends_with(']') {
        &s[1..s.len()-1]
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
    let toks: Vec<&str> = line.trim().split_whitespace().collect();

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

        // Purchases
        ["BUY", "PLANE", model, aid] => Ok(Command::BuyPlane {
            model: model.to_string(),
            airport: aid.parse().map_err(|_| "bad airport id")?,
        }),

        // Exit
        ["EXIT"] => Ok(Command::Exit),

        // Queries
        ["SHOW", "CASH"] => Ok(Command::ShowCash),
        ["SHOW", "TIME"] => Ok(Command::ShowTime),

        // Time control
        ["ADVANCE", n] => Ok(Command::Advance {
            hours: n.parse().map_err(|_| "bad time n")?,
        }),

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

        ["LOAD",  "ORDERS", orders, "ON", plane_id] => {
            let order_vec = parse_id_list(orders)?;
            let plane = plane_id
            .parse::<usize>()
            .map_err(|_| "bad plane id")?;
            
            Ok(Command::LoadOrders { orders: order_vec, plane })
        }

        other => Err(format!("Unrecognized command: {:?}", other)),
    }
}
