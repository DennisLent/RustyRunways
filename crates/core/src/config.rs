use serde::{Deserialize, Serialize};

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
}

fn default_cash() -> f32 {
    1_000_000.0
}
fn default_generate_orders() -> bool {
    true
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
