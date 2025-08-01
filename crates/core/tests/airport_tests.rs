use rusty_runways_core::utils::{
    airplanes::{
        airplane::Airplane,
        models::{AirplaneModel, AirplaneStatus},
    },
    airport::Airport,
    coordinate::Coordinate,
    errors::GameError,
    orders::Order,
};

fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol
}

#[test]
fn generate_deterministic() {
    let a1 = Airport::generate_random(1234, 5);
    let a2 = Airport::generate_random(1234, 5);
    assert_eq!(a1.id, a2.id);
    assert_eq!(a1.name, a2.name);
    assert!(approx_eq(a1.runway_length, a2.runway_length, 1e-4));
    assert!(approx_eq(a1.fuel_price, a2.fuel_price, 1e-4));
    assert!(approx_eq(a1.landing_fee, a2.landing_fee, 1e-4));
    assert!(approx_eq(a1.parking_fee, a2.parking_fee, 1e-4));
    assert!(a1.orders.is_empty() && a2.orders.is_empty());
}

#[test]
fn properties_within_range() {
    let ap = Airport::generate_random(42, 2);

    // runway is in [245, 5500]
    assert!(ap.runway_length >= 245.0 && ap.runway_length <= 5500.0);

    // fuel price is in [0.5, 2.5]
    assert!(ap.fuel_price >= 0.5 && ap.fuel_price <= 2.5);

    // landing fee is in [2.4, 9.0]
    assert!(ap.landing_fee >= 2.4 && ap.landing_fee <= 9.0);

    // parking fee is in [5, 50]
    assert!(ap.parking_fee >= 5.0 && ap.parking_fee <= 50.0);
}

#[test]
fn landing_and_fueling_fee() {
    let mut ap = Airport::generate_random(0, 0);
    ap.landing_fee = 4.0;
    ap.fuel_price = 2.0;

    let home = Coordinate::new(0.0, 0.0);
    let mut plane = Airplane::new(0, AirplaneModel::Atlas, home);

    // landing fee = 4.0 * (mtow/1000)
    let expected_lf = 4.0 * (plane.specs.mtow / 1000.0);
    assert!(approx_eq(ap.landing_fee(&plane), expected_lf, 1e-3));

    // fuel up partially, then fueling_fee = price * (capacity - current_fuel)
    plane.current_fuel = plane.specs.fuel_capacity * 0.3;
    let expected_fuel = 2.0 * (plane.specs.fuel_capacity - plane.current_fuel);
    let fueling_fee = ap.fueling_fee(&plane);

    assert!(approx_eq(fueling_fee, expected_fuel, 1e-3));
}

#[test]
fn generate_orders_counts_and_ids() {
    // fix runway --> know how many to expect
    // with 1000 number_orders is in [15, 24]
    let mut ap = Airport::generate_random(0, 0);
    ap.runway_length = 1000.0;
    let coords = vec![Coordinate::new(0., 0.), Coordinate::new(10., 10.)];
    let mut next_id = 0;
    ap.generate_orders(0, &coords, coords.len(), &mut next_id);

    assert!(ap.orders.len() >= 5 && ap.orders.len() <= 8);

    assert_eq!(next_id, ap.orders.len());

    // check unique, ascending ids
    let ids: Vec<_> = ap.orders.iter().map(|o| o.id).collect();
    for win in ids.windows(2) {
        assert!(win[1] == win[0] + 1);
    }
}

#[test]
fn load_order_and_errors() {
    // set up airport with one order
    let mut ap = Airport::generate_random(0, 0);
    let coords = vec![Coordinate::new(0., 0.), Coordinate::new(5., 5.)];
    let mut next_id = 0;
    ap.generate_orders(0, &coords, coords.len(), &mut next_id);

    let order = ap.orders[0].clone();
    let home = Coordinate::new(0., 0.);
    let mut plane = Airplane::new(0, AirplaneModel::Atlas, home);

    // load should succeed
    ap.load_order(order.id, &mut plane).unwrap();
    assert_eq!(plane.manifest.len(), 1);
    assert!(matches!(plane.status, AirplaneStatus::Loading));

    // airport empty
    assert!(!ap.orders.iter().any(|o| o.id == order.id));

    // cannot load
    let err = ap.load_order(999, &mut plane).unwrap_err();
    assert!(matches!(err, GameError::OrderIdInvalid { id: 999 }));
}

#[test]
fn load_orders_stops_on_error() {
    // set up airport with two orders
    let mut ap = Airport::generate_random(0, 0);
    let coords = vec![Coordinate::new(0., 0.), Coordinate::new(5., 5.)];

    // these are both going to be the same
    let order1 = Order::new(1, 0, 0, &coords, 2);
    let order2 = Order::new(1, 1, 0, &coords, 2);

    ap.orders = vec![order1, order2];

    // ensure at least two
    assert!(ap.orders.len() == 2);
    let begin_length = ap.orders.len();
    let ids: Vec<usize> = vec![0, 1];

    let home = coords[0];

    // make plan that can hold 1 item, but not two
    let mut plane = Airplane::new(0, AirplaneModel::SparrowLight, home);
    plane.specs.payload_capacity = ap.orders[0].weight + 1.0;

    // attempt batch loading
    let result = ap.load_orders(ids.clone(), &mut plane);
    assert!(result.is_err());
    if let Err(GameError::MaxPayloadReached { added_weight, .. }) = result {
        // the failing added_weight must match second order weight
        // that order is now the first element
        assert_eq!(begin_length - 1, ap.orders.len());
        assert_eq!(added_weight, ap.orders[0].weight);
    } else {
        panic!("Expected MaxPayloadReached");
    }
}
