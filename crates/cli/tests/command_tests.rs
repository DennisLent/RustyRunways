use rusty_runways_cli::commands::{Command, parse_command};

#[test]
fn parse_show_airports() {
    let cmd = parse_command("SHOW AIRPORTS").unwrap();
    assert!(matches!(cmd, Command::ShowAirports { with_orders: false }));
}

#[test]
fn parse_load_orders_with_brackets() {
    let cmd = parse_command("LOAD ORDERS [1,2,3] ON 4").unwrap();
    assert!(
        matches!(cmd, Command::LoadOrders { orders, plane } if orders == vec![1,2,3] && plane == 4)
    );
}

#[test]
fn parse_invalid_command() {
    let err = parse_command("DO SOMETHING");
    assert!(err.is_err());
}
