use rusty_runways_core::utils::coordinate::Coordinate;

#[test]
fn coordinate_new_and_update() {
    let mut c = Coordinate::new(10.0, 20.0);
    c.update(5.0, -10.0);
    assert!((c.x - 15.0).abs() < f32::EPSILON);
    assert!((c.y - 10.0).abs() < f32::EPSILON);
}
