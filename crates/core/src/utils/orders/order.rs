use super::cargo::CargoType;
use crate::{events::GameTime, utils::coordinate::Coordinate};
use rand::{
    Rng, SeedableRng,
    distributions::{Distribution, WeightedIndex},
    rngs::StdRng,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

// ---- Cargo defaults ----
pub const DEFAULT_ALPHA: f32 = 0.12;
pub const DEFAULT_BETA: f32 = 0.55;
pub const DEFAULT_MAX_DEADLINE_HOURS: u64 = 96;
pub const DEFAULT_MIN_WEIGHT: f32 = 180.0;
pub const DEFAULT_MAX_WEIGHT: f32 = 650.0;
const VALUE_CAP: f32 = 8_000_000.0;
const MIN_VALUE: f32 = 400.0;
const BASE_TON_KM_RATE: f32 = 160.0;
const REFERENCE_SPEED_KMH: f32 = 520.0;

// ---- Passenger defaults ----
pub const DEFAULT_PASSENGER_MAX_DEADLINE_HOURS: u64 = 48;
pub const DEFAULT_PASSENGER_MIN_COUNT: u32 = 4;
pub const DEFAULT_PASSENGER_MAX_COUNT: u32 = 220;
pub const DEFAULT_PASSENGER_ALPHA: f32 = 0.10;
pub const DEFAULT_PASSENGER_BETA: f32 = 0.40;
pub const DEFAULT_FARE_PER_KM: f32 = 9.5;
const PASSENGER_VALUE_CAP: f32 = 2_500_000.0;
const PASSENGER_MIN_VALUE: f32 = 150.0;

/// Lightweight description of an airport useful for demand generation heuristics.
#[derive(Clone, Copy, Debug)]
pub struct OrderAirportInfo {
    pub id: usize,
    pub runway_length: f32,
    pub coordinate: Coordinate,
}

/// Parameters controlling random cargo order generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGenerationParams {
    pub max_deadline_hours: u64,
    pub min_weight: f32,
    pub max_weight: f32,
    pub alpha: f32,
    pub beta: f32,
}

impl Default for OrderGenerationParams {
    fn default() -> Self {
        OrderGenerationParams {
            max_deadline_hours: DEFAULT_MAX_DEADLINE_HOURS,
            min_weight: DEFAULT_MIN_WEIGHT,
            max_weight: DEFAULT_MAX_WEIGHT,
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
        }
    }
}

/// Parameters controlling random passenger demand generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassengerGenerationParams {
    pub max_deadline_hours: u64,
    pub min_count: u32,
    pub max_count: u32,
    pub alpha: f32,
    pub beta: f32,
    pub fare_per_km: f32,
}

impl Default for PassengerGenerationParams {
    fn default() -> Self {
        PassengerGenerationParams {
            max_deadline_hours: DEFAULT_PASSENGER_MAX_DEADLINE_HOURS,
            min_count: DEFAULT_PASSENGER_MIN_COUNT,
            max_count: DEFAULT_PASSENGER_MAX_COUNT,
            alpha: DEFAULT_PASSENGER_ALPHA,
            beta: DEFAULT_PASSENGER_BETA,
            fare_per_km: DEFAULT_FARE_PER_KM,
        }
    }
}

/// Bundled configuration for both cargo and passenger demand.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DemandGenerationParams {
    pub cargo: OrderGenerationParams,
    pub passengers: PassengerGenerationParams,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunwayClass {
    Small,
    Medium,
    Large,
}

fn classify_runway(length: f32) -> RunwayClass {
    if length < 1_500.0 {
        RunwayClass::Small
    } else if length < 3_000.0 {
        RunwayClass::Medium
    } else {
        RunwayClass::Large
    }
}

fn destination_lambda(class: RunwayClass) -> f32 {
    match class {
        RunwayClass::Small => 600.0,
        RunwayClass::Medium => 1_600.0,
        RunwayClass::Large => 2_800.0,
    }
}

fn base_weight_profile(class: RunwayClass) -> (f32, f32, f32, f32) {
    match class {
        RunwayClass::Small => (20.0, 800.0, 3.0, 1_200.0),
        RunwayClass::Medium => (200.0, 6_000.0, 1.6, 8_000.0),
        RunwayClass::Large => (2_000.0, 20_000.0, 1.1, 22_000.0),
    }
}

fn passenger_profile(class: RunwayClass) -> (u32, u32, f32, u32) {
    match class {
        RunwayClass::Small => (2, 18, 1.9, 30),
        RunwayClass::Medium => (10, 90, 1.5, 140),
        RunwayClass::Large => (40, 220, 1.2, 320),
    }
}

fn handling_buffer(class: RunwayClass) -> f32 {
    match class {
        RunwayClass::Small => 4.0,
        RunwayClass::Medium => 5.0,
        RunwayClass::Large => 6.0,
    }
}

fn slack_range(class: RunwayClass) -> (f32, f32) {
    match class {
        RunwayClass::Small => (1.25, 1.9),
        RunwayClass::Medium => (1.35, 2.1),
        RunwayClass::Large => (1.45, 2.3),
    }
}

fn controlling_class(origin: RunwayClass, dest: RunwayClass) -> RunwayClass {
    use RunwayClass::*;
    if origin == Small || dest == Small {
        Small
    } else if origin == Medium || dest == Medium {
        Medium
    } else {
        Large
    }
}

fn chord_distance(a: Coordinate, b: Coordinate) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    ((dx * dx) + (dy * dy)).sqrt().max(1.0)
}

fn choose_destination(
    rng: &mut StdRng,
    origin: &OrderAirportInfo,
    airports: &[OrderAirportInfo],
) -> OrderAirportInfo {
    let lambda = destination_lambda(classify_runway(origin.runway_length)).max(1.0);
    let mut weights = Vec::new();
    let mut candidates = Vec::new();

    for info in airports.iter() {
        if info.id == origin.id {
            continue;
        }
        let distance = chord_distance(origin.coordinate, info.coordinate);
        let weight = (-(distance / lambda)).exp().max(1e-6);
        weights.push(weight);
        candidates.push(*info);
    }

    if candidates.is_empty() {
        return *origin;
    }

    match WeightedIndex::new(&weights) {
        Ok(dist) => {
            let idx = dist.sample(rng);
            candidates[idx]
        }
        Err(_) => {
            let idx = rng.gen_range(0..candidates.len());
            candidates[idx]
        }
    }
}

fn sample_weight(
    rng: &mut StdRng,
    origin_class: RunwayClass,
    dest_class: RunwayClass,
    params: &OrderGenerationParams,
) -> f32 {
    let class = controlling_class(origin_class, dest_class);
    let (base_min, base_max, skew, tail_max) = base_weight_profile(class);

    let mut min_w = params.min_weight.max(base_min);
    let mut max_w = params.max_weight.min(base_max);
    if min_w >= max_w {
        min_w = params.min_weight;
        max_w = params.max_weight.max(min_w + 1.0);
    }

    let range = max_w - min_w;
    let rand = rng.gen_range(0.0_f32..=1.0_f32);
    let mut weight = min_w + range * rand.powf(skew);

    if class == RunwayClass::Small && rng.gen_bool(0.1) {
        let extended_max = params.max_weight.min(tail_max.max(max_w));
        let extended_range = (extended_max - min_w).max(1.0);
        let tail_rand = rng.gen_range(0.0_f32..=1.0_f32);
        weight = min_w + extended_range * tail_rand.powf(2.0_f32);
    }

    weight.clamp(params.min_weight, params.max_weight)
}

fn sample_passenger_count(
    rng: &mut StdRng,
    origin_class: RunwayClass,
    dest_class: RunwayClass,
    params: &PassengerGenerationParams,
) -> u32 {
    let class = controlling_class(origin_class, dest_class);
    let (base_min, base_max, skew, tail_max) = passenger_profile(class);

    let mut min_c = params.min_count.max(base_min);
    let mut max_c = params.max_count.min(base_max);
    if min_c >= max_c {
        min_c = params.min_count;
        max_c = params.max_count.max(min_c + 1);
    }

    let range = (max_c - min_c).max(1);
    let rand = rng.gen_range(0.0_f32..=1.0_f32);
    let mut count = min_c + ((range as f32) * rand.powf(skew)).round() as u32;

    if class == RunwayClass::Small && rng.gen_bool(0.08) {
        let extended_max = params.max_count.min(tail_max.max(max_c));
        let extended_range = (extended_max - min_c).max(1);
        let tail_rand = rng.gen_range(0.0_f32..=1.0_f32);
        count = min_c + ((extended_range as f32) * tail_rand.powf(2.0_f32)).round() as u32;
    }

    count.clamp(params.min_count, params.max_count)
}

fn compute_deadline(
    rng: &mut StdRng,
    distance_km: f32,
    origin_class: RunwayClass,
    max_deadline_hours: u64,
) -> (u64, f32, f32) {
    let travel_hours = (distance_km / REFERENCE_SPEED_KMH).max(0.1);
    let buffer = handling_buffer(origin_class);
    let (slack_min, slack_max) = slack_range(origin_class);
    let slack = rng.gen_range(slack_min..=slack_max);

    let mut deadline = ((travel_hours + buffer) * slack).ceil() as u64;
    let min_deadline = ((travel_hours + buffer).ceil() as u64).max(1);
    if deadline < min_deadline {
        deadline = min_deadline;
    }
    if deadline > max_deadline_hours {
        deadline = max_deadline_hours;
    }

    (deadline, travel_hours, buffer)
}

fn compute_cargo_value(
    weight: f32,
    distance_km: f32,
    travel_hours: f32,
    buffer: f32,
    deadline_hours: u64,
    cargo_type: CargoType,
    params: &OrderGenerationParams,
) -> f32 {
    let distance_km = distance_km.max(1.0);
    let weight_tons = (weight / 1_000.0).max(0.05);
    let base_value = distance_km * weight_tons * BASE_TON_KM_RATE;

    let schedule_ratio = ((travel_hours + buffer) / deadline_hours as f32).clamp(0.0, 1.0);
    let urgency_multiplier = 1.0 + params.beta * (1.0 - schedule_ratio);

    let (min_price, max_price) = cargo_type.price_range();
    let cargo_multiplier = ((min_price + max_price) / 2.0 / 10.0).clamp(0.5, 5.0);

    let distance_multiplier = 1.0 + params.alpha * (distance_km / 1_500.0).clamp(0.0, 3.0);

    let value = base_value * urgency_multiplier * cargo_multiplier * distance_multiplier;
    value.clamp(MIN_VALUE, VALUE_CAP).round()
}

fn compute_passenger_value(
    count: u32,
    distance_km: f32,
    travel_hours: f32,
    buffer: f32,
    deadline_hours: u64,
    params: &PassengerGenerationParams,
) -> f32 {
    let distance_km = distance_km.max(1.0);
    let base_value = distance_km * count as f32 * params.fare_per_km;

    let schedule_ratio = ((travel_hours + buffer) / deadline_hours as f32).clamp(0.0, 1.0);
    let urgency_multiplier = 1.0 + params.beta * (1.0 - schedule_ratio);
    let distance_multiplier = 1.0 + params.alpha * (distance_km / 1_500.0).clamp(0.0, 3.0);

    let value = base_value * urgency_multiplier * distance_multiplier;
    value
        .clamp(PASSENGER_MIN_VALUE, PASSENGER_VALUE_CAP)
        .round()
}

/// The payload stored within an [`Order`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "payload_kind", rename_all = "snake_case")]
pub enum OrderPayload {
    Cargo {
        #[serde(alias = "name")]
        cargo_type: CargoType,
        weight: f32,
    },
    Passengers {
        #[serde(alias = "passenger_count")]
        count: u32,
    },
}

impl OrderPayload {
    pub fn cargo_type(&self) -> Option<CargoType> {
        match self {
            OrderPayload::Cargo { cargo_type, .. } => Some(*cargo_type),
            OrderPayload::Passengers { .. } => None,
        }
    }

    pub fn weight(&self) -> Option<f32> {
        match self {
            OrderPayload::Cargo { weight, .. } => Some(*weight),
            OrderPayload::Passengers { .. } => None,
        }
    }

    pub fn passenger_count(&self) -> Option<u32> {
        match self {
            OrderPayload::Passengers { count } => Some(*count),
            OrderPayload::Cargo { .. } => None,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            OrderPayload::Cargo { .. } => "cargo",
            OrderPayload::Passengers { .. } => "passengers",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: usize, // Global unique id
    pub payload: OrderPayload,
    pub value: f32,
    pub deadline: GameTime,
    pub origin_id: usize,
    pub destination_id: usize,
}

impl Order {
    pub fn new_cargo(
        seed: u64,
        order_id: usize,
        origin_airport_id: usize,
        airports: &[OrderAirportInfo],
        params: &OrderGenerationParams,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let cargo_count = CargoType::iter().count();
        let cargo_type = CargoType::iter()
            .nth(rng.gen_range(0..cargo_count))
            .unwrap();

        let origin = airports
            .iter()
            .find(|info| info.id == origin_airport_id)
            .copied()
            .expect("origin airport must exist in airport list");

        let destination = choose_destination(&mut rng, &origin, airports);
        let distance = chord_distance(origin.coordinate, destination.coordinate);

        let origin_class = classify_runway(origin.runway_length);
        let dest_class = classify_runway(destination.runway_length);
        let weight = sample_weight(&mut rng, origin_class, dest_class, params);

        let (deadline, travel_hours, buffer) =
            compute_deadline(&mut rng, distance, origin_class, params.max_deadline_hours);
        let value = compute_cargo_value(
            weight,
            distance,
            travel_hours,
            buffer,
            deadline,
            cargo_type,
            params,
        );

        Order {
            id: order_id,
            payload: OrderPayload::Cargo { cargo_type, weight },
            value,
            deadline,
            origin_id: origin_airport_id,
            destination_id: destination.id,
        }
    }

    pub fn new_passenger(
        seed: u64,
        order_id: usize,
        origin_airport_id: usize,
        airports: &[OrderAirportInfo],
        params: &PassengerGenerationParams,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let origin = airports
            .iter()
            .find(|info| info.id == origin_airport_id)
            .copied()
            .expect("origin airport must exist in airport list");

        let destination = choose_destination(&mut rng, &origin, airports);
        let distance = chord_distance(origin.coordinate, destination.coordinate);

        let origin_class = classify_runway(origin.runway_length);
        let dest_class = classify_runway(destination.runway_length);
        let count = sample_passenger_count(&mut rng, origin_class, dest_class, params);

        let (deadline, travel_hours, buffer) =
            compute_deadline(&mut rng, distance, origin_class, params.max_deadline_hours);
        let value =
            compute_passenger_value(count, distance, travel_hours, buffer, deadline, params);

        Order {
            id: order_id,
            payload: OrderPayload::Passengers { count },
            value,
            deadline,
            origin_id: origin_airport_id,
            destination_id: destination.id,
        }
    }

    /// Return the cargo weight if this is a cargo order.
    ///
    /// Returns
    /// - `Some(weight_kg)` for cargo orders.
    /// - `None` for passenger orders.
    pub fn cargo_weight(&self) -> Option<f32> {
        self.payload.weight()
    }

    /// Return the passenger count if this is a passenger order.
    pub fn passenger_count(&self) -> Option<u32> {
        self.payload.passenger_count()
    }

    /// Return the cargo type if this is a cargo order.
    pub fn cargo_type(&self) -> Option<CargoType> {
        self.payload.cargo_type()
    }

    /// Return `true` if this is a passenger order.
    pub fn is_passenger(&self) -> bool {
        matches!(self.payload, OrderPayload::Passengers { .. })
    }
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn runway_classification() {
        assert_eq!(classify_runway(800.0), RunwayClass::Small);
        assert_eq!(classify_runway(1_800.0), RunwayClass::Medium);
        assert_eq!(classify_runway(3_500.0), RunwayClass::Large);
    }

    #[test]
    fn weight_sampling_respects_bounds() {
        let params = OrderGenerationParams::default();
        let mut rng = StdRng::seed_from_u64(7);
        let w = sample_weight(&mut rng, RunwayClass::Small, RunwayClass::Small, &params);
        assert!(w >= params.min_weight && w <= params.max_weight);
    }

    #[test]
    fn passenger_sampling_respects_bounds() {
        let params = PassengerGenerationParams::default();
        let mut rng = StdRng::seed_from_u64(11);
        let c = sample_passenger_count(&mut rng, RunwayClass::Small, RunwayClass::Small, &params);
        assert!(c >= params.min_count && c <= params.max_count);
    }

    #[test]
    fn deadline_never_zero() {
        let params = OrderGenerationParams::default();
        let mut rng = StdRng::seed_from_u64(3);
        let (deadline, _, _) = compute_deadline(
            &mut rng,
            50.0,
            RunwayClass::Small,
            params.max_deadline_hours,
        );
        assert!(deadline >= 1);
    }

    #[test]
    fn compute_value_positive() {
        let params = OrderGenerationParams::default();
        let value = compute_cargo_value(500.0, 200.0, 1.0, 4.0, 12, CargoType::Food, &params);
        assert!((MIN_VALUE..=VALUE_CAP).contains(&value));
    }

    #[test]
    fn compute_passenger_value_positive() {
        let params = PassengerGenerationParams::default();
        let value = compute_passenger_value(60, 450.0, 2.0, 3.5, 18, &params);
        assert!((PASSENGER_MIN_VALUE..=PASSENGER_VALUE_CAP).contains(&value));
    }

    #[test]
    fn choose_destination_fallback_handles_tiny_weights() {
        let airports = [
            OrderAirportInfo {
                id: 0,
                runway_length: 1_000.0,
                coordinate: Coordinate::new(0.0, 0.0),
            },
            OrderAirportInfo {
                id: 1,
                runway_length: 3_000.0,
                coordinate: Coordinate::new(1_000_000.0, 0.0),
            },
        ];
        let params = OrderGenerationParams::default();
        let order = Order::new_cargo(123, 1, 0, &airports, &params);
        assert_eq!(order.destination_id, 1);
    }

    #[test]
    fn sample_weight_produces_tail_for_small_airports() {
        let params = OrderGenerationParams::default();
        let mut rng = StdRng::seed_from_u64(0);
        let mut saw_tail = false;
        let threshold = params.max_weight * 0.9;
        for _ in 0..50 {
            let weight = sample_weight(&mut rng, RunwayClass::Small, RunwayClass::Small, &params);
            if weight >= threshold {
                saw_tail = true;
                break;
            }
        }
        assert!(
            saw_tail,
            "expected at least one extended-tail sample above {:.1} kg",
            threshold
        );
    }

    #[test]
    fn compute_value_caps_when_enormous() {
        let params = OrderGenerationParams::default();
        let value = compute_cargo_value(
            25_000.0,
            50_000.0,
            80.0,
            6.0,
            24,
            CargoType::Pharmaceuticals,
            &params,
        );
        assert_eq!(value, VALUE_CAP);
    }

    #[test]
    fn compute_passenger_value_caps_when_enormous() {
        let params = PassengerGenerationParams::default();
        let value = compute_passenger_value(400, 8_000.0, 12.0, 7.0, 40, &params);
        assert_eq!(value, PASSENGER_VALUE_CAP);
    }
}
