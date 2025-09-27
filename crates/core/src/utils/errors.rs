use serde::{Deserialize, Serialize};
use std::fmt;
use strsim::levenshtein;
use strum::IntoEnumIterator;

use crate::utils::{
    airplanes::models::{AirplaneModel, AirplaneStatus},
    coordinate::Coordinate,
};

/// Errors surfaced by the simulation and API operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameError {
    OutOfRange {
        distance: f32,
        range: f32,
    },
    RunwayTooShort {
        required: f32,
        available: f32,
    },
    MaxPayloadReached {
        current_capacity: f32,
        maximum_capacity: f32,
        added_weight: f32,
    },
    PassengerCapacityReached {
        current_capacity: u32,
        maximum_capacity: u32,
        added_passengers: u32,
    },
    PayloadTypeUnsupported {
        plane_model: String,
        payload: String,
    },
    OrderIdInvalid {
        id: usize,
    },
    PlaneIdInvalid {
        id: usize,
    },
    AirportIdInvalid {
        id: usize,
    },
    AirportLocationInvalid {
        location: Coordinate,
    },
    PlaneNotAtAirport {
        plane_id: usize,
    },
    PlaneNotReady {
        plane_state: AirplaneStatus,
    },
    InsufficientFunds {
        have: f32,
        need: f32,
    },
    InsufficientFuel {
        have: f32,
        need: f32,
    },
    UnknownModel {
        input: String,
        suggestion: Option<String>,
    },
    NoCargo,
    SameAirport,
    InvalidCommand {
        msg: String,
    },
    InvalidConfig {
        msg: String,
    },
}

impl GameError {
    /// Heuristic helper: suggest an airplane model similar to the given input.
    ///
    /// Parameters
    /// - `input`: User-provided model name (case-insensitive).
    ///
    /// Returns
    /// - `Some(String)` with a suggested model if the edit distance is small.
    /// - `None` otherwise.
    fn suggest_model(input: &str) -> Option<String> {
        let lower = input.to_lowercase();
        let mut best: Option<(usize, String)> = None;
        for model in AirplaneModel::iter() {
            let name = format!("{:?}", model);
            let dist = levenshtein(&lower, &name.to_lowercase());
            match &best {
                Some((best_dist, _)) if *best_dist <= dist => {}
                _ => best = Some((dist, name.clone())),
            }
        }
        best.and_then(|(dist, name)| if dist <= 3 { Some(name) } else { None })
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::OutOfRange { distance, range } => {
                write!(
                    f,
                    "Distance {:.2} is outside of the airplane range {:.2}",
                    distance, range
                )
            }
            GameError::RunwayTooShort {
                required,
                available,
            } => {
                write!(
                    f,
                    "Airplane requires at least {:.2} m of runway. Destination has a length of {:.2}",
                    required, available
                )
            }
            GameError::MaxPayloadReached {
                current_capacity,
                maximum_capacity,
                added_weight,
            } => {
                write!(
                    f,
                    "Cannot load order of weight {:.2}. Airplane capacity: {:.2}. Current Capacity: {:.2}",
                    added_weight, maximum_capacity, current_capacity
                )
            }
            GameError::PassengerCapacityReached {
                current_capacity,
                maximum_capacity,
                added_passengers,
            } => {
                write!(
                    f,
                    "Cannot board {} passengers. Seats available: {}. Currently occupied: {}",
                    added_passengers, maximum_capacity, current_capacity
                )
            }
            GameError::PayloadTypeUnsupported {
                plane_model,
                payload,
            } => {
                write!(f, "Plane {} cannot carry {} payloads", plane_model, payload)
            }
            GameError::OrderIdInvalid { id } => {
                write!(f, "Order with id {:?} does not exist", id)
            }
            GameError::PlaneIdInvalid { id } => {
                write!(f, "Plan with id {:?} does not exist", id)
            }
            GameError::PlaneNotAtAirport { plane_id } => {
                write!(f, "Plane {} is not located at any known airport", plane_id)
            }
            GameError::AirportIdInvalid { id } => {
                write!(f, "Airport with id {} does not exist", id)
            }
            GameError::AirportLocationInvalid { location } => {
                write!(
                    f,
                    "No airport found at coordinate ({:.2}, {:.2})",
                    location.x, location.y
                )
            }
            GameError::InsufficientFunds { have, need } => {
                write!(
                    f,
                    "Insufficient funds. Need: ${:.2}. Currently have: ${:.2}",
                    need, have
                )
            }
            GameError::InsufficientFuel { have, need } => {
                write!(
                    f,
                    "Insufficient fuel. Need: {:.2}L. Currently have: {:.2}L",
                    need, have
                )
            }
            GameError::UnknownModel { input, suggestion } => {
                let sug = suggestion
                    .clone()
                    .or_else(|| GameError::suggest_model(input));
                if let Some(s) = sug {
                    write!(f, "`{}` doesn't exist. Did you mean `{}`?", input, s)
                } else {
                    write!(f, "`{}` doesn't exist.", input)
                }
            }
            GameError::PlaneNotReady { plane_state } => {
                write!(f, "Airplane not ready. Current status: {:?}", plane_state)
            }
            GameError::NoCargo => {
                write!(f, "No cargo to unload")
            }
            GameError::SameAirport => {
                write!(f, "Cannot fly to the airport the plane is currently at")
            }
            GameError::InvalidCommand { msg } => {
                write!(f, "{}", msg)
            }
            GameError::InvalidConfig { msg } => {
                write!(f, "Invalid config: {}", msg)
            }
        }
    }
}
