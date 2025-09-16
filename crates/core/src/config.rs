use crate::utils::orders::order::{
    DEFAULT_ALPHA, DEFAULT_BETA, DEFAULT_MAX_DEADLINE_HOURS, DEFAULT_MAX_WEIGHT,
    DEFAULT_MIN_WEIGHT, OrderGenerationParams,
};
use serde::{Deserialize, Serialize};

pub const DEFAULT_RESTOCK_CYCLE_HOURS: u64 = DEFAULT_MAX_DEADLINE_HOURS;
pub const DEFAULT_FUEL_INTERVAL_HOURS: u64 = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Optional seed to keep deterministic behavior for generated pieces
    #[serde(default)]
    pub seed: Option<u64>,
    /// Starting cash for the player
    #[serde(default = "default_cash")]
    pub starting_cash: f32,
    /// Whether to auto-generate orders based on airports and seed
    #[serde(default = "default_generate_orders")]
    pub generate_orders: bool,
    /// Explicit airports to load into the map
    pub airports: Vec<AirportConfig>,
    /// Optional gameplay tuning parameters
    #[serde(default)]
    pub gameplay: GameplayConfig,
}

fn default_cash() -> f32 {
    1_000_000.0
}
fn default_generate_orders() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameplayConfig {
    pub restock_cycle_hours: u64,
    pub fuel_interval_hours: u64,
    pub orders: OrderTuning,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        GameplayConfig {
            restock_cycle_hours: DEFAULT_RESTOCK_CYCLE_HOURS,
            fuel_interval_hours: DEFAULT_FUEL_INTERVAL_HOURS,
            orders: OrderTuning::default(),
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

impl From<OrderTuning> for OrderGenerationParams {
    fn from(value: OrderTuning) -> Self {
        OrderGenerationParams {
            max_deadline_hours: value.max_deadline_hours,
            min_weight: value.min_weight,
            max_weight: value.max_weight,
            alpha: value.alpha,
            beta: value.beta,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportConfig {
    pub id: usize,
    pub name: String,
    pub location: Location,
    /// meters
    pub runway_length_m: f32,
    /// $/L
    pub fuel_price_per_l: f32,
    /// $ per ton of MTOW
    pub landing_fee_per_ton: f32,
    /// $ per hour
    pub parking_fee_per_hour: f32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Location {
    pub x: f32,
    pub y: f32,
}
