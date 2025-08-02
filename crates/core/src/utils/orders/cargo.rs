use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumIter, PartialEq)]
pub enum CargoType {
    Electronics,
    Furniture,
    Food,
    Machines,
    Clothing,
    Pharmaceuticals,
    Automotive,
    Chemicals,
    PaperGoods,
    RubberDucks,
    LiveAlpacas,
    GiantBalloons,
    HauntedMirrors,
    SingingFish,
    TimeMachines,
    DiscoBalls,
    NitroFuel,
    QuantumWidgets,
}

impl CargoType {
    /// Return the (min, max) price per kg for this cargo type
    /// Allows us to ensure that some items are more expensive for prioritization
    pub fn price_range(&self) -> (f32, f32) {
        match self {
            CargoType::PaperGoods
            | CargoType::Furniture
            | CargoType::RubberDucks
            | CargoType::GiantBalloons
            | CargoType::DiscoBalls => (0.50, 3.00),

            CargoType::Food | CargoType::LiveAlpacas | CargoType::SingingFish => (2.00, 10.00),

            CargoType::Clothing | CargoType::Automotive | CargoType::Electronics => (5.00, 20.00),

            CargoType::Chemicals | CargoType::NitroFuel => (10.00, 50.00),

            CargoType::Pharmaceuticals | CargoType::TimeMachines | CargoType::QuantumWidgets => {
                (50.00, 500.00)
            }

            CargoType::Machines | CargoType::HauntedMirrors => (20.00, 100.00),
        }
    }
}
