use RustyRunways::utils::{
    coordinate::Coordinate,
    orders::{
        Order,
        cargo::CargoType,
        order::{ALPHA, BETA, MAX_DEADLINE},
    },
};
use strum::IntoEnumIterator;

fn approx_le(a: f32, b: f32, tol: f32) -> bool {
    a <= b + tol
}

fn approx_ge(a: f32, b: f32, tol: f32) -> bool {
    a + tol >= b
}

#[test]
fn iter_cargo_types() {
    let variants: Vec<_> = CargoType::iter().collect();
    // 18 variants
    assert_eq!(variants.len(), 18, "Found {:?}, want 18", variants);
}

#[test]
fn price_ranges() {
    for ct in CargoType::iter() {
        let (min, max) = ct.price_range();
        assert!(min > 0.0, "{}: min must be > 0", ct as usize);
        assert!(max > min, "{}: max must exceed min", ct as usize);
    }
}

#[test]
fn match_price_ranges() {
    // cheap
    assert_eq!(CargoType::PaperGoods.price_range(), (0.50, 3.00));
    assert_eq!(CargoType::RubberDucks.price_range(), (0.50, 3.00));

    // mid
    assert_eq!(CargoType::Food.price_range(), (2.00, 10.00));
    assert_eq!(CargoType::Clothing.price_range(), (5.00, 20.00));

    // expensive
    assert_eq!(CargoType::Pharmaceuticals.price_range(), (50.00, 500.00));

    // silly
    assert_eq!(CargoType::HauntedMirrors.price_range(), (20.00, 100.00));
}

#[test]
fn new_order_is_deterministic() {
    let coords = vec![
        Coordinate::new(0.0, 0.0),
        Coordinate::new(1000.0, 0.0),
        Coordinate::new(0.0, 1000.0),
    ];
    let o1 = Order::new(42, 7, 0, &coords, coords.len());
    let o2 = Order::new(42, 7, 0, &coords, coords.len());
    // same seed & order_id => same everything
    assert_eq!(o1.id, o2.id);
    assert_eq!(o1.name, o2.name);
    assert_eq!(o1.origin_id, o2.origin_id);
    assert_eq!(o1.destination_id, o2.destination_id);
    assert_eq!(o1.deadline, o2.deadline);
    assert!(approx_le(o1.weight, o2.weight, 1e-6) && approx_ge(o1.weight, o2.weight, 1e-6));
    assert!(approx_le(o1.value, o2.value, 1e-3) && approx_ge(o1.value, o2.value, 1e-3));
}

#[test]
fn cannot_arrive_at_origin() {
    let coords = vec![Coordinate::new(0.0, 0.0), Coordinate::new(1.0, 1.0)];
    let origin = 1;
    let order = Order::new(7, 3, origin, &coords, coords.len());
    assert_ne!(order.destination_id, origin);
    assert!(order.destination_id < coords.len());
}

#[test]
fn deadline_weight_check() {
    let coords = vec![Coordinate::new(0., 0.), Coordinate::new(10., 10.)];
    for seed in 0..5 {
        let o = Order::new(seed, seed as usize, 0, &coords, coords.len());

        // deadline in [1, 30]
        assert!((1..=MAX_DEADLINE * 24).contains(&o.deadline));

        // weight in [100, 20000]
        assert!(o.weight >= 100.0 && o.weight <= 20_000.0);
    }
}

#[test]
fn value_of_order_check() {
    // (0,0) -> (10,10) = approx 14.14

    let coords = vec![Coordinate::new(0., 0.), Coordinate::new(10., 10.)];
    let seed = 123;
    let o = Order::new(seed, 1, 0, &coords, coords.len());
    let (min_p, max_p) = o.name.price_range();

    // base = weight * price_per_kg
    let base_min = o.weight * min_p;
    let base_max = o.weight * max_p;

    // distance factor = 1 + 0.5*(distance/10000)
    let dist = (10.0f32).hypot(10.0);
    let dist_factor = 1.0 + ALPHA * (dist / 10000.0);

    // time factor = 1 + 0.7*((d0-deadline)/d0)
    let time_factor = 1.0
        + BETA * (((MAX_DEADLINE * 24) as f32 - (o.deadline as f32)) / (MAX_DEADLINE * 24) as f32);

    // overall value needs to be in [base_min, base_max] * dist_factor * time_factor
    let lower = (base_min * dist_factor * time_factor).floor();
    let upper = (base_max * dist_factor * time_factor).ceil();

    assert!(
        approx_ge(o.value, lower, 1.0),
        "value {} < lower {}",
        o.value,
        lower
    );
    assert!(
        approx_le(o.value, upper, 1.0),
        "value {} > upper {}",
        o.value,
        upper
    );
}
