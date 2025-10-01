use rusty_runways_core::utils::{
    coordinate::Coordinate,
    orders::{
        Order,
        cargo::CargoType,
        order::{
            DEFAULT_MAX_DEADLINE_HOURS, DEFAULT_MAX_WEIGHT, DEFAULT_MIN_WEIGHT,
            DEFAULT_PASSENGER_MAX_COUNT, DEFAULT_PASSENGER_MIN_COUNT, OrderAirportInfo,
            OrderGenerationParams, PassengerGenerationParams,
        },
    },
};
use strum::IntoEnumIterator;

fn approx_le(a: f32, b: f32, tol: f32) -> bool {
    a <= b + tol
}

fn approx_ge(a: f32, b: f32, tol: f32) -> bool {
    a + tol >= b
}

fn sample_airports() -> Vec<OrderAirportInfo> {
    vec![
        OrderAirportInfo {
            id: 0,
            runway_length: 1_200.0,
            coordinate: Coordinate::new(0.0, 0.0),
        },
        OrderAirportInfo {
            id: 1,
            runway_length: 2_400.0,
            coordinate: Coordinate::new(1_000.0, 0.0),
        },
        OrderAirportInfo {
            id: 2,
            runway_length: 3_800.0,
            coordinate: Coordinate::new(0.0, 1_400.0),
        },
    ]
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
    let airports = sample_airports();
    let params = OrderGenerationParams::default();
    let o1 = Order::new_cargo(42, 7, 0, &airports, &params);
    let o2 = Order::new_cargo(42, 7, 0, &airports, &params);
    // same seed & order_id => same everything
    assert_eq!(o1.id, o2.id);
    assert_eq!(o1.cargo_type(), o2.cargo_type());
    assert_eq!(o1.origin_id, o2.origin_id);
    assert_eq!(o1.destination_id, o2.destination_id);
    assert_eq!(o1.deadline, o2.deadline);
    let w1 = o1.cargo_weight().unwrap();
    let w2 = o2.cargo_weight().unwrap();
    assert!(approx_le(w1, w2, 1e-6) && approx_ge(w1, w2, 1e-6));
    assert!(approx_le(o1.value, o2.value, 1e-3) && approx_ge(o1.value, o2.value, 1e-3));
}

#[test]
fn cannot_arrive_at_origin() {
    let airports = vec![
        OrderAirportInfo {
            id: 0,
            runway_length: 1_200.0,
            coordinate: Coordinate::new(0.0, 0.0),
        },
        OrderAirportInfo {
            id: 1,
            runway_length: 1_300.0,
            coordinate: Coordinate::new(1.0, 1.0),
        },
    ];
    let origin = 1;
    let params = OrderGenerationParams::default();
    let order = Order::new_cargo(7, 3, origin, &airports, &params);
    assert_ne!(order.destination_id, origin);
    assert!(order.destination_id < airports.len());
}

#[test]
fn weight_respects_runway_class() {
    let airports = sample_airports();
    let params = OrderGenerationParams::default();
    let order = Order::new_cargo(11, 2, 0, &airports, &params);
    let weight = order.cargo_weight().unwrap();
    assert!(
        weight <= 1_500.0,
        "small airports should generate lighter freight"
    );
}

#[test]
fn respects_config_weight_bounds() {
    let airports = sample_airports();
    let params = OrderGenerationParams {
        min_weight: 5_000.0,
        max_weight: 6_000.0,
        ..OrderGenerationParams::default()
    };
    let order = Order::new_cargo(25, 9, 2, &airports, &params);
    let weight = order.cargo_weight().unwrap();
    assert!((5_000.0..=6_000.0).contains(&weight));
}

#[test]
fn deadlines_and_value_scale_with_distance() {
    let near_airports = vec![
        OrderAirportInfo {
            id: 0,
            runway_length: 2_500.0,
            coordinate: Coordinate::new(0.0, 0.0),
        },
        OrderAirportInfo {
            id: 1,
            runway_length: 2_500.0,
            coordinate: Coordinate::new(120.0, 0.0),
        },
    ];
    let far_airports = vec![
        OrderAirportInfo {
            id: 0,
            runway_length: 2_500.0,
            coordinate: Coordinate::new(0.0, 0.0),
        },
        OrderAirportInfo {
            id: 1,
            runway_length: 2_500.0,
            coordinate: Coordinate::new(2_400.0, 0.0),
        },
    ];
    let params = OrderGenerationParams::default();
    let near = Order::new_cargo(99, 0, 0, &near_airports, &params);
    let far = Order::new_cargo(99, 0, 0, &far_airports, &params);

    assert!(far.deadline >= near.deadline);
    assert!(far.value >= near.value);
}

#[test]
fn deadlines_clamped_to_config_max() {
    let airports = vec![
        OrderAirportInfo {
            id: 0,
            runway_length: 3_600.0,
            coordinate: Coordinate::new(0.0, 0.0),
        },
        OrderAirportInfo {
            id: 1,
            runway_length: 3_200.0,
            coordinate: Coordinate::new(8_000.0, 0.0),
        },
    ];
    let params = OrderGenerationParams {
        max_deadline_hours: 48,
        ..OrderGenerationParams::default()
    };
    let order = Order::new_cargo(5, 1, 0, &airports, &params);
    assert!(order.deadline <= 48);
    assert!(order.deadline >= 1);
}

#[test]
fn basic_bounds_still_hold() {
    let airports = sample_airports();
    for seed in 0..5 {
        let params = OrderGenerationParams::default();
        let o = Order::new_cargo(seed, seed as usize, 0, &airports, &params);
        assert!((1..=DEFAULT_MAX_DEADLINE_HOURS).contains(&o.deadline));
        let wt = o.cargo_weight().unwrap();
        assert!((DEFAULT_MIN_WEIGHT..=DEFAULT_MAX_WEIGHT).contains(&wt));
        assert!(o.value >= 0.0);
    }
}

#[test]
fn passenger_counts_respect_bounds() {
    let airports = sample_airports();
    let params = PassengerGenerationParams {
        min_count: 30,
        max_count: 42,
        ..PassengerGenerationParams::default()
    };
    let order = Order::new_passenger(17, 5, 1, &airports, &params);
    let count = order.passenger_count().unwrap();
    assert!((30..=42).contains(&count));
    assert!(order.is_passenger());
}

#[test]
fn passenger_deadlines_clamped() {
    let airports = sample_airports();
    let params = PassengerGenerationParams {
        max_deadline_hours: 24,
        ..PassengerGenerationParams::default()
    };
    let order = Order::new_passenger(9, 2, 0, &airports, &params);
    assert!(order.deadline <= 24);
    assert!(order.deadline >= 1);
    let count = order.passenger_count().unwrap();
    assert!((DEFAULT_PASSENGER_MIN_COUNT..=DEFAULT_PASSENGER_MAX_COUNT).contains(&count));
}
