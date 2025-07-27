use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::utils::coordinate::Coordinate;

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter)]
pub enum AirplaneModel {
    SparrowLight,  // Small prop plane
    FalconJet,     // Light biz jet
    CometRegional, // Regional turbofan
    Atlas,         // Narrow‑body jet
    TitanHeavy,    // Wide‑body freighter
    Goliath,       // Super‑heavy lift
    Zephyr,        // Long‑range twin‑aisle
    Lightning,     // Supersonic small jet
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AirplaneSpecs {
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
    /// Purchase price
    pub purchase_price: f32,
}

impl AirplaneModel {
    /// Return the full spec bundle for each model, including its purchase price.
    pub fn specs(&self) -> AirplaneSpecs {
        match self {
            AirplaneModel::SparrowLight => AirplaneSpecs {
                mtow: 5_000.0,
                cruise_speed: 250.0,
                fuel_capacity: 200.0,
                fuel_consumption: 30.0,
                operating_cost: 300.0,
                payload_capacity: 500.0,
                purchase_price: 200_000.0, // 200k
            },
            AirplaneModel::FalconJet => AirplaneSpecs {
                mtow: 8_000.0,
                cruise_speed: 800.0,
                fuel_capacity: 2_000.0,
                fuel_consumption: 250.0,
                operating_cost: 1_500.0,
                payload_capacity: 1_500.0,
                purchase_price: 1_500_000.0, // 1.5M
            },
            AirplaneModel::CometRegional => AirplaneSpecs {
                mtow: 20_000.0,
                cruise_speed: 700.0,
                fuel_capacity: 5_000.0,
                fuel_consumption: 600.0,
                operating_cost: 3_000.0,
                payload_capacity: 5_000.0,
                purchase_price: 10_000_000.0, // 10M
            },
            AirplaneModel::Atlas => AirplaneSpecs {
                mtow: 40_000.0,
                cruise_speed: 750.0,
                fuel_capacity: 12_000.0,
                fuel_consumption: 1_500.0,
                operating_cost: 6_000.0,
                payload_capacity: 15_000.0,
                purchase_price: 30_000_000.0, // 30M
            },
            AirplaneModel::TitanHeavy => AirplaneSpecs {
                mtow: 100_000.0,
                cruise_speed: 650.0,
                fuel_capacity: 20_000.0,
                fuel_consumption: 3_000.0,
                operating_cost: 10_000.0,
                payload_capacity: 50_000.0,
                purchase_price: 60_000_000.0, // 60M
            },
            AirplaneModel::Goliath => AirplaneSpecs {
                mtow: 200_000.0,
                cruise_speed: 550.0,
                fuel_capacity: 40_000.0,
                fuel_consumption: 6_000.0,
                operating_cost: 20_000.0,
                payload_capacity: 100_000.0,
                purchase_price: 120_000_000.0, // 120M
            },
            AirplaneModel::Zephyr => AirplaneSpecs {
                mtow: 50_000.0,
                cruise_speed: 900.0,
                fuel_capacity: 25_000.0,
                fuel_consumption: 1_200.0,
                operating_cost: 8_000.0,
                payload_capacity: 25_000.0,
                purchase_price: 50_000_000.0, // 50M
            },
            AirplaneModel::Lightning => AirplaneSpecs {
                mtow: 15_000.0,
                cruise_speed: 1_800.0,
                fuel_capacity: 5_000.0,
                fuel_consumption: 1_000.0,
                operating_cost: 12_000.0,
                payload_capacity: 2_000.0,
                purchase_price: 80_000_000.0, // 80M
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AirplaneStatus {
    Parked,
    Refueling,
    Maintenance,
    Loading,
    Unloading,
    InTransit { destination: Coordinate },
}
