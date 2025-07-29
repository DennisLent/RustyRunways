
#[derive(Debug)]
pub enum Command {
    ShowAirports { with_orders: bool },
    ShowAirport  { id: usize, with_orders: bool },
    ShowAirplanes,
    ShowAirplane { id: usize },
    BuyPlane     { model: String, airport: usize },
    LoadOrder    { orders: Vec<usize>, plane: usize },
    UnloadOrder  { order: usize, plane: usize },
    UnloadAll    { plane: usize },
    DepartPlane  { plane: usize, dest: usize },
    HoldPlane    { plane: usize },
    Advance      { hours: u64 },
    ShowCash,
    ShowTime,
    Exit,
}

pub fn parse_command(line: &str) -> Result<Command, String> {

    let toks: Vec<&str> = line.trim().split_whitespace().collect();

    match toks.as_slice() {
        ["SHOW", "AIRPORTS"] =>
            Ok(Command::ShowAirports { with_orders: false }),

        ["SHOW", "AIRPORTS", "WITH", "ORDERS"] =>
            Ok(Command::ShowAirports { with_orders: true }),

        ["SHOW", "AIRPORTS", id] =>
            Ok(Command::ShowAirport { 
                id: id.parse().map_err(|_| "bad airport id")?,
                with_orders: false 
            }),

        ["SHOW", "AIRPORTS", id, "WITH", "ORDERS"] =>
            Ok(Command::ShowAirport {
                id: id.parse().map_err(|_| "bad airport id")?,
                with_orders: true
            }),

        ["SHOW", "PLANES"] =>
            Ok(Command::ShowAirplanes),

        ["SHOW", "PLANES", pid] =>
            Ok(Command::ShowAirplane {
                id: pid.parse().map_err(|_| "bad plane id")?
            }),

        ["BUY", "PLANE", model, aid] =>
            Ok(Command::BuyPlane {
                model: model.to_string(),
                airport: aid.parse().map_err(|_| "bad airport id")?
            }),

        ["EXIT"] => Ok(Command::Exit),
        ["QUERY", "CASH"] => Ok(Command::ShowCash),
        ["QUERY", "TIME"] => Ok(Command::ShowTime),

        other => Err(format!("Unrecognized command: {:?}", other)),
    }
}

