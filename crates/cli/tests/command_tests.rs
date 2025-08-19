use rusty_runways_cli::commands::{Command, parse_command};

#[test]
fn parse_show_airports() {
    let cmd = parse_command("SHOW AIRPORTS").unwrap();
    assert!(matches!(cmd, Command::ShowAirports { with_orders: false }));
}

#[test]
fn parse_show_airports_with_orders() {
    let cmd = parse_command("SHOW AIRPORTS WITH ORDERS").unwrap();
    assert!(matches!(cmd, Command::ShowAirports { with_orders: true }));
}

#[test]
fn parse_load_orders_with_brackets() {
    let cmd = parse_command("LOAD ORDERS [1,2,3] ON 4").unwrap();
    assert!(
        matches!(cmd, Command::LoadOrders { orders, plane } if orders == vec![1,2,3] && plane == 4)
    );
}

#[test]
fn parse_load_orders_without_brackets() {
    let cmd = parse_command("LOAD ORDERS 1,2,3 ON 4").unwrap();
    assert!(
        matches!(cmd, Command::LoadOrders { orders, plane } if orders == vec![1,2,3] && plane == 4)
    );
}

#[test]
fn parse_invalid_command() {
    let err = parse_command("DO SOMETHING");
    assert!(err.is_err());
}

#[test]
fn parse_maintenance_command() {
    let cmd = parse_command("MAINTENANCE 3").unwrap();
    assert!(matches!(cmd, Command::Maintenance { plane_id: 3 }));
}

#[test]
fn parse_unload_all_command() {
    let cmd = parse_command("UNLOAD ALL FROM 2").unwrap();
    assert!(matches!(cmd, Command::UnloadAll { plane } if plane == 2));
}

#[test]
fn parse_empty_advances_one_hour() {
    let cmd = parse_command("").unwrap();
    assert!(matches!(cmd, Command::Advance { hours: 1 }));
}

#[test]
fn parse_show_airport_with_orders() {
    let cmd = parse_command("SHOW AIRPORTS 3 WITH ORDERS").unwrap();
    assert!(matches!(cmd, Command::ShowAirport { id: 3, with_orders: true }));
}

#[test]
fn parse_buy_plane_command() {
    let cmd = parse_command("BUY PLANE CESSNA 2").unwrap();
    assert!(matches!(cmd, Command::BuyPlane { model, airport } if model == "CESSNA" && airport == 2));
}

#[test]
fn parse_depart_plane_command() {
    let cmd = parse_command("DEPART PLANE 4 1").unwrap();
    assert!(matches!(cmd, Command::DepartPlane { plane, dest } if plane == 4 && dest == 1));
}

#[test]
fn parse_load_orders_missing_on_errors() {
    assert!(parse_command("LOAD ORDERS 1,2 3").is_err());
}
