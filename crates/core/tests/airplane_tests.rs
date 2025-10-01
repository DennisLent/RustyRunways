use rusty_runways_core::utils::airplanes::models::AirplaneStatus;
use rusty_runways_core::utils::airplanes::{airplane::Airplane, models::AirplaneModel};
use rusty_runways_core::utils::errors::GameError;
use rusty_runways_core::utils::{
    airport::Airport,
    coordinate::Coordinate,
    orders::{CargoType, Order, order::OrderPayload},
};
use strum::IntoEnumIterator;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-2
}

fn sample_airport(runway: f32, x: f32, y: f32) -> (Airport, Coordinate) {
    let mut ap = Airport::generate_random(0, 0);
    ap.runway_length = runway;
    (ap, Coordinate::new(x, y))
}

fn make_cargo_order(id: usize, weight: f32, value: f32, dest: usize) -> Order {
    Order {
        id,
        payload: OrderPayload::Cargo {
            cargo_type: CargoType::Electronics,
            weight,
        },
        value,
        deadline: 10,
        origin_id: 0,
        destination_id: dest,
    }
}

fn make_passenger_order(id: usize, count: u32, value: f32, dest: usize) -> Order {
    Order {
        id,
        payload: OrderPayload::Passengers { count },
        value,
        deadline: 12,
        origin_id: 0,
        destination_id: dest,
    }
}

#[test]
fn iter_models() {
    // By default 10 configs
    let variants: Vec<_> = AirplaneModel::iter().collect();
    assert_eq!(variants.len(), 10);
    for m in AirplaneModel::iter() {
        assert!(variants.contains(&m));
    }
}

#[test]
fn spec_table() {
    let sparrow = AirplaneModel::SparrowLight.specs();
    assert_eq!(sparrow.mtow, 5_200.0);
    assert_eq!(sparrow.cruise_speed, 260.0);
    assert_eq!(sparrow.fuel_capacity, 240.0);
    assert_eq!(sparrow.fuel_consumption, 35.0);
    assert_eq!(sparrow.operating_cost, 340.0);
    assert_eq!(sparrow.payload_capacity, 1_200.0);
    assert_eq!(sparrow.passenger_capacity, 6);
    assert_eq!(sparrow.purchase_price, 240_000.0);

    let titan = AirplaneModel::TitanHeavy.specs();
    assert_eq!(titan.mtow, 110_000.0);
    assert_eq!(titan.cruise_speed, 670.0);
    assert!(titan.fuel_capacity > 20_000.0);
    assert!(titan.payload_capacity > 50_000.0);
    assert_eq!(titan.passenger_capacity, 0);
}

#[test]
fn runway_length() {
    // For SparrowLight:
    // derived from updated specs (takeoff/landing based on new cruise speed)
    let sparrow = AirplaneModel::SparrowLight.specs();
    let req = sparrow.min_runway_length;
    assert!(
        approx_eq(req, 440.75616),
        "Expected approx 440.76m, got {:.6}m",
        req
    );

    // For faster jets, need longer runway
    let lightning = AirplaneModel::Lightning.specs();
    assert!(lightning.min_runway_length > sparrow.min_runway_length);

    // Heavier jets need longer runway
    let goliath = AirplaneModel::Goliath.specs();
    assert!(goliath.min_runway_length > sparrow.min_runway_length);
}

#[test]
fn new_plane_fueled_and_empty() {
    let home = Coordinate::new(100.0, 100.0);
    let plane = Airplane::new(42, AirplaneModel::FalconJet, home);
    assert_eq!(plane.id, 42);
    assert_eq!(plane.location, home);
    assert!(approx_eq(plane.current_fuel, plane.specs.fuel_capacity));
    assert_eq!(plane.current_payload, 0.0);
    assert_eq!(plane.current_passengers, 0);
    assert_eq!(plane.manifest.len(), 0);
    assert!(matches!(plane.status, AirplaneStatus::Parked));
}

#[test]
fn distance_endurance_and_range_check() {
    let home = Coordinate::new(0.0, 0.0);
    let target = Coordinate::new(3.0, 4.0);
    let plane = Airplane::new(0, AirplaneModel::SparrowLight, home);
    let dist = plane.distance_to(&target);
    assert!(approx_eq(dist, 5.0));
    let hours = plane.endurance_hours();
    assert!(hours > 0.0);
    assert!(approx_eq(
        plane.max_range(),
        hours * plane.specs.cruise_speed
    ));
}

#[test]
fn can_fly_to_detects_oob_and_runway() {
    // plane with almost no fuel
    let home = Coordinate::new(0.0, 0.0);
    let mut plane = Airplane::new(0, AirplaneModel::SparrowLight, home);
    plane.current_fuel = 1.0;

    // Cannot reach this
    let (far_ap, far_coord) = sample_airport(10000.0, 1000.0, 0.0);
    let err = plane.can_fly_to(&far_ap, &far_coord).unwrap_err();
    assert!(matches!(err, GameError::OutOfRange { .. }));

    // Cannot land here
    plane.current_fuel = plane.specs.fuel_capacity;
    let (short_ap, short_coord) = sample_airport(100.0, 10.0, 0.0);
    let err2 = plane.can_fly_to(&short_ap, &short_coord).unwrap_err();
    assert!(matches!(err2, GameError::RunwayTooShort { .. }));

    // Can reach and land
    let (good_ap, good_coord) = sample_airport(1000.0, 10.0, 0.0);
    assert!(plane.can_fly_to(&good_ap, &good_coord).is_ok());
}

#[test]
fn load_and_unload() {
    let home = Coordinate::new(0.0, 0.0);
    let mut plane = Airplane::new(0, AirplaneModel::Atlas, home);

    // order too large
    let big = make_cargo_order(1, plane.specs.payload_capacity + 1.0, 1000.0, 0);
    assert!(matches!(
        plane.load_order(big.clone()),
        Err(GameError::MaxPayloadReached { .. })
    ));

    // order fits
    let small = make_cargo_order(2, plane.specs.payload_capacity - 1.0, 1000.0, 0);
    plane.load_order(small.clone()).unwrap();
    assert_eq!(plane.manifest.len(), 1);
    assert_eq!(plane.current_payload, small.cargo_weight().unwrap());
    assert!(matches!(plane.status, AirplaneStatus::Loading));

    // unload
    let delivered = plane.unload_all();
    assert_eq!(delivered.len(), 1);
    assert_eq!(plane.manifest.len(), 0);
    assert_eq!(plane.current_payload, 0.0);
    assert_eq!(plane.current_passengers, 0);
    assert!(matches!(plane.status, AirplaneStatus::Unloading));
}

#[test]
fn passenger_capacity_enforced() {
    let home = Coordinate::new(0.0, 0.0);
    let mut cargo_plane = Airplane::new(0, AirplaneModel::BisonFreighter, home);
    let pax_order = make_passenger_order(3, 10, 5_000.0, 1);
    assert!(matches!(
        cargo_plane.load_order(pax_order.clone()),
        Err(GameError::PayloadTypeUnsupported { .. })
    ));

    let mut mixed_plane = Airplane::new(1, AirplaneModel::TrailblazerCombi, home);
    mixed_plane.load_order(pax_order.clone()).unwrap();
    assert_eq!(mixed_plane.current_passengers, 10);
    let delivered = mixed_plane.unload_all();
    assert_eq!(delivered.len(), 1);
    assert_eq!(mixed_plane.current_passengers, 0);
}

#[test]
fn passenger_overload_rejected() {
    let home = Coordinate::new(0.0, 0.0);
    let mut plane = Airplane::new(2, AirplaneModel::FalconJet, home);
    let too_many = make_passenger_order(4, plane.specs.passenger_capacity + 5, 8_000.0, 1);
    assert!(matches!(
        plane.load_order(too_many),
        Err(GameError::PassengerCapacityReached { .. })
    ));
}

#[test]
fn flying_consumes_status() {
    let home = Coordinate::new(0.0, 0.0);
    let (ap, coords) = sample_airport(1000.0, 100.0, 0.0);
    let mut plane = Airplane::new(0, AirplaneModel::SparrowLight, home);

    // check we are flying
    let before_fuel = plane.current_fuel;
    plane.consume_flight_fuel(&ap, &coords).unwrap();

    // location unchanged durinng the duration of flight
    assert_eq!(plane.location, home);

    // fuel decreased
    assert!(plane.current_fuel < before_fuel);
}

#[test]
fn refuel_check() {
    let home = Coordinate::new(0.0, 0.0);
    let mut plane = Airplane::new(0, AirplaneModel::FalconJet, home);
    plane.current_fuel = 0.0;
    plane.refuel();
    assert!(approx_eq(plane.current_fuel, plane.specs.fuel_capacity));
    assert!(matches!(plane.status, AirplaneStatus::Refueling));
}
