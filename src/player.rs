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
    pub fn new(starting_cash: f32, map: &Map) -> Self {
        let mut player = Player {
            cash: starting_cash,
            fleet_size: 0,
            fleet: Vec::new(),
            orders_delivered: 0,
        };

        let (minimum_distance, start_index) = map.min_distance();

        let best_model = AirplaneModel::iter()
            .filter(|model| {
                let specs = model.specs();
                let max_range = (specs.fuel_capacity / specs.fuel_consumption) * specs.cruise_speed;
                max_range > minimum_distance
            })
            .min_by(|a, b| {
                let purchasing_price_1 = a.specs().purchase_price;
                let purchasing_price_2 = b.specs().purchase_price;
                purchasing_price_1.partial_cmp(&purchasing_price_2).unwrap()
            })
            .unwrap_or(AirplaneModel::CometRegional);

        let (_, starting_coordinates) = map.airports[start_index];
        let new_plane = Airplane::new(0, best_model, starting_coordinates);
        player.fleet.push(new_plane);
        player.fleet_size += 1;

        player
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

    /// Records that the player has delivered an order
    pub fn record_delivery(&mut self) {
        self.orders_delivered += 1;
    }
}
