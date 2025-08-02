use rusty_runways_core::utils::coordinate::Coordinate;

#[test]
fn coordinate_creation() {
    let c = Coordinate::new(1.0, -2.5);
    assert_eq!(c.x, 1.0);
    assert_eq!(c.y, -2.5);
}

#[test]
fn coordinate_update_changes_position() {
    let mut c = Coordinate::new(0.0, 0.0);
    c.update(3.0, -4.0);
    assert_eq!(c.x, 3.0);
    assert_eq!(c.y, -4.0);
}
