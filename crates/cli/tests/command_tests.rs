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
    assert!(matches!(
        cmd,
        Command::ShowAirport {
            id: 3,
            with_orders: true
        }
    ));
}

#[test]
fn parse_buy_plane_command() {
    let cmd = parse_command("BUY PLANE CESSNA 2").unwrap();
    assert!(
        matches!(cmd, Command::BuyPlane { model, airport } if model == "CESSNA" && airport == 2)
    );
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

#[test]
fn parse_advance_command() {
    let cmd = parse_command("ADVANCE 2").unwrap();
    assert!(matches!(cmd, Command::Advance { hours: 2 }));
}

#[test]
fn parse_show_planes_commands() {
    let cmd = parse_command("SHOW PLANES").unwrap();
    assert!(matches!(cmd, Command::ShowAirplanes));
    let cmd = parse_command("SHOW PLANES 1").unwrap();
    assert!(matches!(cmd, Command::ShowAirplane { id: 1 }));
}

#[test]
fn parse_show_distances_command() {
    let cmd = parse_command("SHOW DISTANCES 2").unwrap();
    assert!(matches!(cmd, Command::ShowDistances { plane_id: 2 }));
}

#[test]
fn parse_refuel_plane_command() {
    let cmd = parse_command("REFUEL PLANE 3").unwrap();
    assert!(matches!(cmd, Command::Refuel { plane: 3 }));
}

#[test]
fn parse_unload_order_commands() {
    let cmd = parse_command("UNLOAD ORDER 5 FROM 2").unwrap();
    assert!(matches!(cmd, Command::UnloadOrder { order, plane } if order == 5 && plane == 2));
    let cmd = parse_command("UNLOAD ORDERS [1,2] ON 3").unwrap();
    assert!(
        matches!(cmd, Command::UnloadOrders { orders, plane } if orders == vec![1,2] && plane == 3)
    );
}

#[test]
fn parse_hold_plane_command() {
    let cmd = parse_command("HOLD PLANE 4").unwrap();
    assert!(matches!(cmd, Command::HoldPlane { plane: 4 }));
}

#[test]
fn parse_show_info_commands() {
    assert!(matches!(
        parse_command("SHOW CASH").unwrap(),
        Command::ShowCash
    ));
    assert!(matches!(
        parse_command("SHOW TIME").unwrap(),
        Command::ShowTime
    ));
    assert!(matches!(
        parse_command("SHOW STATS").unwrap(),
        Command::ShowStats
    ));
}

#[test]
fn parse_save_and_load_commands() {
    let cmd = parse_command("SAVE testgame").unwrap();
    assert!(matches!(cmd, Command::SaveGame { name } if name == "testgame"));
    let cmd = parse_command("LOAD testgame").unwrap();
    assert!(matches!(cmd, Command::LoadGame { name } if name == "testgame"));
}

#[test]
fn parse_advance_invalid_number_errors() {
    assert!(parse_command("ADVANCE two").is_err());
}
