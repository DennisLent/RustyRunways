use crate::utils::orders::{
    cargo::CargoType,
    order::{
        DEFAULT_ALPHA, DEFAULT_BETA, DEFAULT_MAX_DEADLINE_HOURS, DEFAULT_MAX_WEIGHT,
        DEFAULT_MIN_WEIGHT, OrderGenerationParams,
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
}

impl Default for OrdersGameplay {
    fn default() -> Self {
        OrdersGameplay {
            regenerate: true,
            generate_initial: true,
            tuning: OrderTuning::default(),
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
pub struct ManualOrderConfig {
    pub cargo: CargoType,
    pub weight: f32,
    pub value: f32,
    pub deadline_hours: u64,
    pub destination_id: usize,
}
