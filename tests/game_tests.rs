use RustyRunways::events::Event;
use RustyRunways::game::Game;
use RustyRunways::utils::orders::{CargoType, Order};

#[test]
fn schedule_and_process_flight_arrival() {
    let mut game = Game::new(1, Some(2), 0.0);
    // add a plane from player's fleet into game.airplanes
    let plane = game.player.fleet[0].clone();
    game.airplanes.push(plane);

    let order = Order {
        name: CargoType::Electronics,
        weight: 100.0,
        value: 500.0,
        deadline: 1,
        origin_id: 0,
        destination_id: 1,
    };
    game.airplanes[0].load_order(order.clone()).unwrap();

    let dest_coord = game.map.airports[1].1;
    game.schedule(
        5,
        Event::FlightArrival {
            plane: 0,
            airport_coord: dest_coord,
        },
    );

    assert!(game.tick_event());
    assert_eq!(game.time, 5);
    assert_eq!(game.airplanes[0].location.x, dest_coord.x);
    assert!(game.airplanes[0].manifest.is_empty());
    assert_eq!(game.player.orders_delivered, 1);
    assert_eq!(game.player.cash, 500.0);
}

#[test]
fn order_deadline_removes_order() {
    let mut game = Game::new(2, Some(2), 0.0);
    let orders_before = game.map.airports[0].0.orders.len();
    assert!(orders_before > 0);
    game.schedule(
        1,
        Event::OrderDeadline {
            airport: 0,
            order_index: 0,
        },
    );
    game.tick_event();
    let orders_after = game.map.airports[0].0.orders.len();
    assert_eq!(orders_after + 1, orders_before);
}
