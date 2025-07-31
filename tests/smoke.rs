use RustyRunways::{game::Game, utils::{airplanes::models::{AirplaneModel, AirplaneStatus}, errors::GameError}};
#[test]
fn test_game_new() {
    let game = Game::new(1, Some(5), 1_000_000.0);
    assert_eq!(game.map.num_airports, 5);
    assert_eq!(game.player.cash, 1_000_000.0);
}

#[test]
fn buy_new_plane() {
    let mut game = Game::new(1, Some(5), 10_000_000.0);

    // before
    let cash_before = game.player.cash;
    assert_eq!(cash_before, 10_000_000.0);
    assert_eq!(game.player.fleet_size, 1);

    // Shouldn't fail
    game.buy_plane(&"FalconJet".to_string(), 0).unwrap();

    let cash_after = game.player.cash;
    assert_eq!(cash_after, cash_before - AirplaneModel::FalconJet.specs().purchase_price);
    assert_eq!(game.player.fleet_size, 2);

    let new_plane = game.airplanes.last().unwrap();
    let target_airport = game.map.airports[0].1;
    assert_eq!(new_plane.location, target_airport);

}

#[test]
fn load_check() {
    let mut game = Game::new(1, Some(5), 10_000_000.0);
    
    // airplane CometRegional (capacity of 5T) starts at airport id 2
    //     ID: 2 | AAC at (1964.23, 4279.75) | Runway: 3648m | Fuel: $0.69/L | Parking: $37.05/hr | Landing Fee: $8.94/ton
    //   Orders:
    //     [42] NitroFuel -> AAD | weight: 4726.8kg | value: $128112.00 | deadline: 120hr | destination: 3
    //     [43] GiantBalloons -> AAA | weight: 13249.2kg | value: $43241.00 | deadline: 240hr | destination: 0
    //     [44] Machines -> AAD | weight: 5256.5kg | value: $825077.00 | deadline: 120hr | destination: 3
    //     [45] Pharmaceuticals -> AAD | weight: 16909.3kg | value: $5477570.00 | deadline: 144hr | destination: 3
    //     [46] DiscoBalls -> AAE | weight: 13001.5kg | value: $66964.00 | deadline: 24hr | destination: 4
    //     [47] SingingFish -> AAB | weight: 13018.0kg | value: $180211.00 | deadline: 312hr | destination: 1
    //     [48] Pharmaceuticals -> AAE | weight: 7055.5kg | value: $2624788.00 | deadline: 120hr | destination: 4
    //     [49] Machines -> AAD | weight: 6954.3kg | value: $557918.00 | deadline: 96hr | destination: 3
    //     [50] Chemicals -> AAD | weight: 7138.9kg | value: $243986.00 | deadline: 168hr | destination: 3
    //     [51] RubberDucks -> AAA | weight: 11642.3kg | value: $55126.00 | deadline: 144hr | destination: 0
    //     [52] SingingFish -> AAD | weight: 2313.6kg | value: $30110.00 | deadline: 48hr | destination: 3
    //     [53] Machines -> AAD | weight: 7633.5kg | value: $352625.00 | deadline: 264hr | destination: 3
    //     [54] Pharmaceuticals -> AAD | weight: 12195.9kg | value: $7881711.00 | deadline: 240hr | destination: 3
    //     [55] QuantumWidgets -> AAD | weight: 16284.8kg | value: $10801045.00 | deadline: 120hr | destination: 3
    //     [56] Clothing -> AAB | weight: 13804.3kg | value: $438214.00 | deadline: 168hr | destination: 1
    //     [57] Automotive -> AAE | weight: 8024.6kg | value: $118391.00 | deadline: 336hr | destination: 4
    //     [58] GiantBalloons -> AAA | weight: 15568.7kg | value: $17564.00 | deadline: 216hr | destination: 0
    //     [59] RubberDucks -> AAB | weight: 1377.9kg | value: $5801.00 | deadline: 168hr | destination: 1
    //     [60] RubberDucks -> AAE | weight: 16715.8kg | value: $62171.00 | deadline: 192hr | destination: 4
    //     [61] Furniture -> AAB | weight: 8284.2kg | value: $19652.00 | deadline: 264hr | destination: 1
    //     [62] Food -> AAD | weight: 4450.9kg | value: $58307.00 | deadline: 144hr | destination: 3
    //     [63] PaperGoods -> AAD | weight: 11168.9kg | value: $8138.00 | deadline: 216hr | destination: 3
    //     [64] NitroFuel -> AAD | weight: 3618.6kg | value: $129751.00 | deadline: 264hr | destination: 3
    //     [65] SingingFish -> AAE | weight: 11928.0kg | value: $107058.00 | deadline: 288hr | destination: 4
    //     [66] Electronics -> AAB | weight: 17948.6kg | value: $271000.00 | deadline: 336hr | destination: 1
    //     [67] Chemicals -> AAA | weight: 837.5kg | value: $65046.00 | deadline: 192hr | destination: 0
    //     [68] Clothing -> AAA | weight: 10561.3kg | value: $349043.00 | deadline: 24hr | destination: 0
    //     [69] Clothing -> AAD | weight: 984.0kg | value: $20290.00 | deadline: 312hr | destination: 3
    //     [70] Clothing -> AAB | weight: 12548.4kg | value: $193695.00 | deadline: 216hr | destination: 1
    //     [71] PaperGoods -> AAA | weight: 11690.4kg | value: $56741.00 | deadline: 144hr | destination: 0
    //     [72] NitroFuel -> AAD | weight: 15087.4kg | value: $876159.00 | deadline: 120hr | destination: 3

    // Try loading order 70 (too heavy)
    assert!(matches!(game.load_order(70, 0), Err(GameError::MaxPayloadReached { .. })));
    assert_eq!(game.airplanes[0].status, AirplaneStatus::Parked);

    // Load 67 (works)
    game.load_order(67, 0).unwrap();
    assert_eq!(game.airplanes[0].status, AirplaneStatus::Loading);

    // Skip ahead and check
    game.advance(1);

    // round it because just to be sure
    assert_eq!(game.airplanes[0].current_payload.round(), 837.0);
    assert_eq!(game.airplanes[0].manifest.len(), 1);
    assert_eq!(game.airplanes[0].manifest[0].id, 67);
    assert_eq!(game.airplanes[0].status, AirplaneStatus::Parked);

}

#[test]
fn delivery_cycle() {
    let mut game = Game::new(1, Some(5), 10_000_000.0);

    // load 69 because actually plane cant fly to airport 0
    game.load_order(69, 0).unwrap();
    game.advance(1);

    let before_take_off = game.player.cash;
    // shouldn't fail
    game.depart_plane(0, 3).unwrap();
    assert!(matches!(game.airplanes[0].status, AirplaneStatus::InTransit { .. }));
    let take_off_fee = game.map.airports[2].0.parking_fee * (game.time as f32);
    assert_eq!(game.player.cash, before_take_off - take_off_fee);

    let (hours_remaining, destination) = if let AirplaneStatus::InTransit { hours_remaining, destination } = game.airplanes[0].status {
        (hours_remaining, destination)
    }
    // We would fail before we even get here because of the test earlier
    else {
        unreachable!()
    };

    let before_landing = game.player.cash;
    game.advance(hours_remaining);

    // plane should have arrived right now
    assert_eq!(game.airplanes[0].status, AirplaneStatus::Parked);
    assert_eq!(game.arrival_times[0], game.time);
    assert_eq!(game.airplanes[0].location, game.map.airports[destination].1);
    let landing_fee = (game.airplanes[0].specs.mtow / 1000.0) * game.map.airports[3].0.landing_fee;
    assert_eq!(game.player.cash, before_landing - landing_fee);

    //unload orders
    let before_unload = game.player.cash;
    let order_value = game.airplanes[0].manifest[0].value;
    game.unload_all(0).unwrap();
    assert_eq!(game.airplanes[0].status, AirplaneStatus::Unloading);

    game.advance(1);
    assert_eq!(game.airplanes[0].status, AirplaneStatus::Parked);
    assert_eq!(game.airplanes[0].manifest.is_empty(), true);
    assert_eq!(game.airplanes[0].current_payload, 0.0);
    assert_eq!(game.player.cash, before_unload + order_value);

    //refueling
    // TODO

}
