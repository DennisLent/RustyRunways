use RustyRunways::utils::airplanes::{airplane::Airplane, models::AirplaneModel};
use RustyRunways::utils::{
    coordinate::Coordinate,
    orders::{CargoType, Order},
};

#[test]
fn airplane_load_fly_unload_cycle() {
    let mut plane = Airplane::new(0, AirplaneModel::SparrowLight, Coordinate::new(0.0, 0.0));
    let order = Order {
        name: CargoType::Electronics,
        weight: 100.0,
        value: 1000.0,
        deadline: 1,
        origin_id: 0,
        destination_id: 1,
    };
    plane.load_order(order.clone()).expect("load");
    assert_eq!(plane.current_payload, order.weight);
    assert_eq!(plane.manifest.len(), 1);

    let dest = Coordinate::new(10.0, 0.0);
    assert!(plane.fly_to(dest));
    assert_eq!(plane.location.x, dest.x);
    assert_eq!(plane.location.y, dest.y);

    let delivered = plane.unload_all();
    assert!(plane.manifest.is_empty());
    assert_eq!(delivered.len(), 1);
    assert_eq!(plane.current_payload, 0.0);
}

#[test]
fn airplane_fly_to_fails_without_fuel() {
    let mut plane = Airplane::new(0, AirplaneModel::SparrowLight, Coordinate::new(0.0, 0.0));
    // drain fuel
    plane.current_fuel = 0.0;
    let far = Coordinate::new(1000.0, 0.0);
    assert!(!plane.fly_to(far));
    assert_eq!(plane.location.x, 0.0);
}
