use rusty_runways_core::utils::airport::Airport;

#[test]
fn ensure_base_sets_when_zero() {
    let mut ap = Airport::generate_random(1, 0);
    ap.base_fuel_price = 0.0;
    let price = ap.fuel_price;
    ap.ensure_base_fuel_price();
    assert!((ap.base_fuel_price - price).abs() < f32::EPSILON);
}
