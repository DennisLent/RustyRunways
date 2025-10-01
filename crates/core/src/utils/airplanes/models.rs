use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{events::GameTime, utils::coordinate::Coordinate};

/// The primary mission role an airplane model is optimized for.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AirplaneRole {
    #[default]
    Cargo,
    Passenger,
    Mixed,
}

/// Catalog of available airplane models.
#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, PartialEq)]
pub enum AirplaneModel {
    SparrowLight,     // Small prop plane
    FalconJet,        // Light biz jet
    CometRegional,    // Regional turbofan
    Atlas,            // Narrow‑body jet
    TitanHeavy,       // Wide‑body freighter
    Goliath,          // Super‑heavy lift
    Zephyr,           // Long‑range twin‑aisle
    Lightning,        // Supersonic small jet
    BisonFreighter,   // Medium cargo hauler
    TrailblazerCombi, // High-capacity combi aircraft
}

/// Static performance and economic specifications for an airplane model.
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
    /// Passenger capacity (people)
    #[serde(default)]
    pub passenger_capacity: u32,
    /// Purchase price
    pub purchase_price: f32,
    /// Minimum runway length required (meters)
    pub min_runway_length: f32,
    /// Primary mission role
    #[serde(default)]
    pub role: AirplaneRole,
}

impl AirplaneModel {
    /// Return the full spec bundle for each model, including computed runway requirement.
    ///
    /// Returns
    /// - `AirplaneSpecs`: Structure populated with static and computed properties.
    pub fn specs(&self) -> AirplaneSpecs {
        // base numeric specs
        let (
            mtow,
            cruise_kmh,
            fuel_cap,
            burn_rate,
            op_cost,
            payload_cap,
            passenger_cap,
            purchase_price,
            role,
        ) = match self {
            AirplaneModel::SparrowLight => (
                5_200.0,
                260.0,
                240.0,
                35.0,
                340.0,
                1_200.0,
                6,
                240_000.0,
                AirplaneRole::Mixed,
            ),
            AirplaneModel::FalconJet => (
                8_300.0,
                780.0,
                2_200.0,
                260.0,
                1_600.0,
                600.0,
                12,
                1_700_000.0,
                AirplaneRole::Passenger,
            ),
            AirplaneModel::CometRegional => (
                24_000.0,
                720.0,
                6_000.0,
                620.0,
                3_200.0,
                4_000.0,
                78,
                12_000_000.0,
                AirplaneRole::Passenger,
            ),
            AirplaneModel::Atlas => (
                42_000.0,
                750.0,
                12_500.0,
                1_550.0,
                6_500.0,
                18_000.0,
                68,
                34_000_000.0,
                AirplaneRole::Mixed,
            ),
            AirplaneModel::TitanHeavy => (
                110_000.0,
                670.0,
                22_000.0,
                3_200.0,
                11_000.0,
                55_000.0,
                0,
                68_000_000.0,
                AirplaneRole::Cargo,
            ),
            AirplaneModel::Goliath => (
                210_000.0,
                580.0,
                45_000.0,
                6_500.0,
                22_000.0,
                110_000.0,
                0,
                130_000_000.0,
                AirplaneRole::Cargo,
            ),
            AirplaneModel::Zephyr => (
                82_000.0,
                900.0,
                28_000.0,
                1_450.0,
                9_000.0,
                8_000.0,
                210,
                72_000_000.0,
                AirplaneRole::Passenger,
            ),
            AirplaneModel::Lightning => (
                18_500.0,
                1_800.0,
                5_400.0,
                1_100.0,
                12_000.0,
                1_500.0,
                32,
                88_000_000.0,
                AirplaneRole::Passenger,
            ),
            AirplaneModel::BisonFreighter => (
                28_000.0,
                680.0,
                8_500.0,
                900.0,
                4_800.0,
                20_000.0,
                0,
                18_000_000.0,
                AirplaneRole::Cargo,
            ),
            AirplaneModel::TrailblazerCombi => (
                65_000.0,
                820.0,
                18_000.0,
                1_800.0,
                7_500.0,
                25_000.0,
                120,
                55_000_000.0,
                AirplaneRole::Mixed,
            ),
        };

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
            passenger_capacity: passenger_cap,
            purchase_price,
            min_runway_length,
            role,
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
