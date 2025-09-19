use crate::utils::{
    airplanes::{airplane::Airplane, models::AirplaneModel},
    airport::Airport,
    coordinate::Coordinate,
    errors::GameError,
    map::Map,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Available cash for purchases and operations
    pub cash: f32,
    /// Number of airplanes owned (always kept in sync with `fleet.len()`)
    pub fleet_size: usize,
    /// The active fleet of airplanes
    pub fleet: Vec<Airplane>,
    /// Total orders successfully delivered
    pub orders_delivered: usize,
}

impl Player {
    /// Create a new player and start them out with the most basic airplane possible.
    /// We check the shortest distance and filter by price to check which one to give.
    /// We try to ensure that the starter plane can fly between at least 2 given airports.
    pub fn new(starting_cash: f32, map: &Map) -> Self {
        let (_min_dist, start_idx) = map.min_distance();
        let start_coord = map.airports[start_idx].1;
        let start_runway = map.airports[start_idx].0.runway_length;

        // find all models that can both take off from start AND transit AND land at some other airport
        let candidates = AirplaneModel::iter()
            .filter(|model| {
                let specs = model.specs();
                let max_range = specs.fuel_capacity / specs.fuel_consumption * specs.cruise_speed;

                // start runway long enough?
                if start_runway < specs.min_runway_length {
                    return false;
                }

                // can reach & land at other airport?
                map.airports.iter().any(|(other_airport, other_coord)| {
                    if other_airport.id == start_idx {
                        return false;
                    }

                    let dx = other_coord.x - start_coord.x;
                    let dy = other_coord.y - start_coord.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    dist <= max_range && other_airport.runway_length >= specs.min_runway_length
                })
            })
            .collect::<Vec<_>>();

        // pick the cheapest (fallback to a mid‑tier if none qualify)
        let best_model = candidates
            .into_iter()
            .min_by(|a, b| {
                a.specs()
                    .purchase_price
                    .partial_cmp(&b.specs().purchase_price)
                    .unwrap()
            })
            .unwrap_or(AirplaneModel::CometRegional);

        // assign player new plane
        let (_, start_coord) = map.airports[start_idx];

        Player {
            cash: starting_cash,
            fleet_size: 1,
            fleet: vec![Airplane::new(0, best_model, start_coord)],
            orders_delivered: 0,
        }
    }

    /// Purchase an additional plane of the given model at `home_coord`.
    pub fn buy_plane(
        &mut self,
        model_name: &String,
        airport: &mut Airport,
        home_coord: &Coordinate,
    ) -> Result<(), GameError> {
        // Try to find matching model
        let model = AirplaneModel::iter()
            .find(|m| format!("{:?}", m).eq_ignore_ascii_case(model_name))
            .ok_or(GameError::UnknownModel {
                input: model_name.to_string(),
                suggestion: None,
            })?;

        let specs = model.specs();
        if self.cash < specs.purchase_price {
            return Err(GameError::InsufficientFunds {
                have: self.cash,
                need: specs.purchase_price,
            });
        }
        if specs.min_runway_length > airport.runway_length {
            return Err(GameError::RunwayTooShort {
                required: specs.min_runway_length,
                available: airport.runway_length,
            });
        }
        self.cash -= specs.purchase_price;
        let plane_id = self.fleet_size;
        let plane_coord = Coordinate::new(home_coord.x, home_coord.y);
        let plane = Airplane::new(plane_id, model, plane_coord);
        self.fleet.push(plane);
        self.fleet_size += 1;
        Ok(())
    }

    /// Sell a plane by id, returning the removed airplane and cash refund.
    pub fn sell_plane(&mut self, plane_id: usize) -> Result<(Airplane, f32), GameError> {
        let idx = self
            .fleet
            .iter()
            .position(|plane| plane.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let plane = self.fleet.remove(idx);
        let refund = plane.specs.purchase_price * 0.6;
        self.cash += refund;
        self.fleet_size = self.fleet.len();

        Ok((plane, refund))
    }

    /// Records that the player has delivered an order
    pub fn record_delivery(&mut self) {
        self.orders_delivered += 1;
    }
}
