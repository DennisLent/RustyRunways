use crate::utils::orders::{
    cargo::CargoType,
    order::{
        DEFAULT_ALPHA, DEFAULT_BETA, DEFAULT_FARE_PER_KM, DEFAULT_MAX_DEADLINE_HOURS,
        DEFAULT_MAX_WEIGHT, DEFAULT_MIN_WEIGHT, DEFAULT_PASSENGER_ALPHA, DEFAULT_PASSENGER_BETA,
        DEFAULT_PASSENGER_MAX_COUNT, DEFAULT_PASSENGER_MAX_DEADLINE_HOURS,
        DEFAULT_PASSENGER_MIN_COUNT, OrderGenerationParams, PassengerGenerationParams,
    },
};
use serde::{Deserialize, Serialize};

pub const DEFAULT_RESTOCK_CYCLE_HOURS: u64 = 168;
pub const DEFAULT_FUEL_INTERVAL_HOURS: u64 = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Optional seed to keep deterministic behavior for generated pieces
    #[serde(default)]
    pub seed: Option<u64>,
    /// Starting cash for the player
    #[serde(default = "default_cash")]
    pub starting_cash: f32,
    /// Explicit airports to load into the map
    #[serde(default)]
    pub airports: Vec<AirportConfig>,
    /// Number of airports to generate randomly when `airports` is empty
    #[serde(default)]
    pub num_airports: Option<usize>,
    /// Optional gameplay tuning parameters
    #[serde(default)]
    pub gameplay: GameplayConfig,

    /// Optional airplane catalog configuration. When provided, controls the
    /// available airplane models in the world either by replacing the default
    /// catalog entirely or by adding new models.
    #[serde(default)]
    pub airplanes: Option<AirplanesConfig>,
}

fn default_cash() -> f32 {
    650_000.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameplayConfig {
    pub restock_cycle_hours: u64,
    pub fuel_interval_hours: u64,
    pub orders: OrdersGameplay,
    pub fuel: FuelGameplay,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        GameplayConfig {
            restock_cycle_hours: DEFAULT_RESTOCK_CYCLE_HOURS,
            fuel_interval_hours: DEFAULT_FUEL_INTERVAL_HOURS,
            orders: OrdersGameplay::default(),
            fuel: FuelGameplay::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FuelGameplay {
    /// Elasticity applied when prices adjust (fractional step size per interval)
    pub elasticity: f32,
    /// Lower bound multiplier relative to the base fuel price
    pub min_price_multiplier: f32,
    /// Upper bound multiplier relative to the base fuel price
    pub max_price_multiplier: f32,
}

impl Default for FuelGameplay {
    fn default() -> Self {
        FuelGameplay {
            elasticity: 0.04,
            min_price_multiplier: 0.6,
            max_price_multiplier: 1.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrdersGameplay {
    pub regenerate: bool,
    pub generate_initial: bool,
    #[serde(flatten)]
    pub tuning: OrderTuning,
    #[serde(default)]
    pub passengers: PassengerTuning,
}

impl Default for OrdersGameplay {
    fn default() -> Self {
        OrdersGameplay {
            regenerate: true,
            generate_initial: true,
            tuning: OrderTuning::default(),
            passengers: PassengerTuning::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderTuning {
    pub max_deadline_hours: u64,
    pub min_weight: f32,
    pub max_weight: f32,
    pub alpha: f32,
    pub beta: f32,
}

impl Default for OrderTuning {
    fn default() -> Self {
        OrderTuning {
            max_deadline_hours: DEFAULT_MAX_DEADLINE_HOURS,
            min_weight: DEFAULT_MIN_WEIGHT,
            max_weight: DEFAULT_MAX_WEIGHT,
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PassengerTuning {
    pub max_deadline_hours: u64,
    pub min_count: u32,
    pub max_count: u32,
    pub alpha: f32,
    pub beta: f32,
    pub fare_per_km: f32,
}

impl Default for PassengerTuning {
    fn default() -> Self {
        PassengerTuning {
            max_deadline_hours: DEFAULT_PASSENGER_MAX_DEADLINE_HOURS,
            min_count: DEFAULT_PASSENGER_MIN_COUNT,
            max_count: DEFAULT_PASSENGER_MAX_COUNT,
            alpha: DEFAULT_PASSENGER_ALPHA,
            beta: DEFAULT_PASSENGER_BETA,
            fare_per_km: DEFAULT_FARE_PER_KM,
        }
    }
}

impl From<&OrderTuning> for OrderGenerationParams {
    fn from(value: &OrderTuning) -> Self {
        OrderGenerationParams {
            max_deadline_hours: value.max_deadline_hours,
            min_weight: value.min_weight,
            max_weight: value.max_weight,
            alpha: value.alpha,
            beta: value.beta,
        }
    }
}

impl From<&PassengerTuning> for PassengerGenerationParams {
    fn from(value: &PassengerTuning) -> Self {
        PassengerGenerationParams {
            max_deadline_hours: value.max_deadline_hours,
            min_count: value.min_count,
            max_count: value.max_count,
            alpha: value.alpha,
            beta: value.beta,
            fare_per_km: value.fare_per_km,
        }
    }
}

// ==========================
// Airplane catalog config
// ==========================

/// Strategy for applying user-provided airplane models.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AirplaneCatalogStrategy {
    /// Replace the default airplane catalog entirely.
    Replace,
    /// Add new models to the default catalog.
    #[default]
    Add,
}

/// User-provided airplane model configuration. All fields are required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirplaneModelConfig {
    /// Unique model name (case-insensitive for matching) — e.g. "MyCargo100".
    pub name: String,
    /// Max take‑off weight (kg)
    pub mtow: f32,
    /// Cruise speed (km/h)
    pub cruise_speed: f32,
    /// Fuel tank capacity (liters)
    pub fuel_capacity: f32,
    /// Fuel burn rate (liters per hour)
    pub fuel_consumption: f32,
    /// Operating cost ($ per hour)
    pub operating_cost: f32,
    /// Cargo payload capacity (kg)
    pub payload_capacity: f32,
    /// Passenger capacity (people)
    pub passenger_capacity: u32,
    /// Purchase price ($)
    pub purchase_price: f32,
    /// Minimum runway length required (meters)
    pub min_runway_length: f32,
    /// Primary mission role ("cargo", "passenger", or "mixed")
    pub role: crate::utils::airplanes::models::AirplaneRole,
}

/// Optional airplane configuration block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AirplanesConfig {
    /// Whether to replace the default catalog or add to it.
    pub strategy: AirplaneCatalogStrategy,
    /// List of fully-specified models.
    pub models: Vec<AirplaneModelConfig>,
}

impl Default for AirplanesConfig {
    fn default() -> Self {
        AirplanesConfig {
            strategy: AirplaneCatalogStrategy::Add,
            models: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportConfig {
    pub id: usize,
    pub name: String,
    #[serde(default)]
    pub location: Option<Location>,
    /// meters
    #[serde(default)]
    pub runway_length_m: Option<f32>,
    /// $/L
    #[serde(default)]
    pub fuel_price_per_l: Option<f32>,
    /// $ per ton of MTOW
    #[serde(default)]
    pub landing_fee_per_ton: Option<f32>,
    /// $ per hour
    #[serde(default)]
    pub parking_fee_per_hour: Option<f32>,
    /// Static orders that should exist at the start of the game
    #[serde(default)]
    pub orders: Vec<ManualOrderConfig>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub struct Location {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ManualOrderConfig {
    Cargo {
        cargo: CargoType,
        weight: f32,
        value: f32,
        deadline_hours: u64,
        destination_id: usize,
    },
    Passengers {
        passengers: u32,
        value: f32,
        deadline_hours: u64,
        destination_id: usize,
    },
}
