use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{events::GameTime, utils::coordinate::Coordinate};

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, PartialEq)]
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
    /// Minimum runway length required (meters)
    pub min_runway_length: f32,
}

impl AirplaneModel {
    /// Return the full spec bundle for each model, including computed runway requirement.
    pub fn specs(&self) -> AirplaneSpecs {
        // base numeric specs
        let base = match self {
            AirplaneModel::SparrowLight => (5_000.0, 250.0, 200.0, 30.0, 300.0, 500.0, 200_000.0),
            AirplaneModel::FalconJet => (
                8_000.0,
                800.0,
                2_000.0,
                250.0,
                1_500.0,
                1_500.0,
                1_500_000.0,
            ),
            AirplaneModel::CometRegional => (
                20_000.0,
                700.0,
                5_000.0,
                600.0,
                3_000.0,
                5_000.0,
                10_000_000.0,
            ),
            AirplaneModel::Atlas => (
                40_000.0,
                750.0,
                12_000.0,
                1_500.0,
                6_000.0,
                15_000.0,
                30_000_000.0,
            ),
            AirplaneModel::TitanHeavy => (
                100_000.0,
                650.0,
                20_000.0,
                3_000.0,
                10_000.0,
                50_000.0,
                60_000_000.0,
            ),
            AirplaneModel::Goliath => (
                200_000.0,
                550.0,
                40_000.0,
                6_000.0,
                20_000.0,
                100_000.0,
                120_000_000.0,
            ),
            AirplaneModel::Zephyr => (
                50_000.0,
                900.0,
                25_000.0,
                1_200.0,
                8_000.0,
                25_000.0,
                50_000_000.0,
            ),
            AirplaneModel::Lightning => (
                15_000.0,
                1_800.0,
                5_000.0,
                1_000.0,
                12_000.0,
                2_000.0,
                80_000_000.0,
            ),
        };
        let (mtow, cruise_kmh, fuel_cap, burn_rate, op_cost, payload_cap, purchase_price) = base;

        // Cruise speed as m/s
        let cruise_ms: f32 = cruise_kmh * 1000.0 / 3600.0;

        // Say takeoff speed ~ 0.65 * cruise
        let takeoff_speed: f32 = 0.65 * cruise_ms;

        // Assume acceleration on run (~2.5 m/s2)
        let accel = 2.5;
        let takeoff_dist = takeoff_speed.powi(2) / (2.0 * accel);

        // Assume deceleration ~4 m/s2
        let decel = 4.0;
        let landing_dist = takeoff_speed.powi(2) / (2.0 * decel);

        // Runway length requirement is the larger of the two
        let min_runway_length = takeoff_dist.max(landing_dist);

        AirplaneSpecs {
            mtow,
            cruise_speed: cruise_kmh,
            fuel_capacity: fuel_cap,
            fuel_consumption: burn_rate,
            operating_cost: op_cost,
            payload_capacity: payload_cap,
            purchase_price,
            min_runway_length,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AirplaneStatus {
    Parked,
    Refueling,
    Maintenance,
    Loading,
    Unloading,
    InTransit {
        hours_remaining: GameTime,
        destination: usize,
        origin: Coordinate,
        total_hours: GameTime,
    },
    Broken,
}
