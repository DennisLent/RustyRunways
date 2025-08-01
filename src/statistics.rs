use serde::{Deserialize, Serialize};

/// Records all of the main stats for the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub day: u64,
    pub income: f32,
    pub expenses: f32,
    pub net_cash: f32,
    pub fleet_size: usize,
    pub total_deliveries: usize,
}
