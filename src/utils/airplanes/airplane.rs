use super::models::{AirplaneModel, AirplaneSpecs, AirplaneStatus};
use crate::utils::{airport::Airport, coordinate::Coordinate, errors::GameError, orders::Order};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// An airplane operating between airports, tracked by precise coordinates
pub struct Airplane {
    pub id: usize,
    pub model: AirplaneModel,
    pub specs: AirplaneSpecs,
    pub status: AirplaneStatus,
    /// Current location in the same coordinate space as airports
    pub location: Coordinate,
    pub current_fuel: f32,
    pub current_payload: f32,
    pub manifest: Vec<Order>,
}

impl Airplane {
    /// Create a fresh airplane, parked and fueled up at `home_airport_coordinates`.
    pub fn new(id: usize, model: AirplaneModel, home_airport_coordinates: Coordinate) -> Self {
        let specs = model.specs();
        Airplane {
            id,
            model,
            specs,
            status: AirplaneStatus::Parked,
            location: home_airport_coordinates,
            current_fuel: specs.fuel_capacity,
            current_payload: 0.0,
            manifest: Vec::new(),
        }
    }

    /// Euclidean distance from current location to `target_coordinates`
    pub fn distance_to(&self, target_coordinates: &Coordinate) -> f32 {
        let dx = self.location.x - target_coordinates.x;
        let dy = self.location.y - target_coordinates.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// How many hours can we fly on current fuel?
    pub fn endurance_hours(&self) -> f32 {
        self.current_fuel / self.specs.fuel_consumption
    }

    /// Maximum range (km) before refuel
    pub fn max_range(&self) -> f32 {
        self.endurance_hours() * self.specs.cruise_speed
    }

    /// Check if an airplane can reach the airport and if it can land there based on the runway requirement
    pub fn can_fly_to(
        &self,
        airport: &Airport,
        airport_coords: &Coordinate,
    ) -> Result<(), GameError> {
        let distance = self.distance_to(airport_coords);

        // Cannot go this far
        if distance > self.max_range() {
            return Err(GameError::OutOfRange {
                distance: distance,
                range: self.max_range(),
            });
        }
        // Cannot land on this airport
        if airport.runway_length < self.specs.min_runway_length {
            return Err(GameError::RunwayTooShort {
                required: self.specs.min_runway_length,
                available: airport.runway_length,
            });
        }

        return Ok(());
    }

    /// Load an order if it fits; returns Err(order) if too heavy
    pub fn load_order(&mut self, order: Order) -> Result<(), GameError> {
        if self.current_payload + order.weight <= self.specs.payload_capacity {
            self.current_payload += order.weight;
            self.manifest.push(order);
            self.status = AirplaneStatus::Loading;
            Ok(())
        } else {
            Err(GameError::MaxPayloadReached {
                current_capacity: self.current_payload,
                maximum_capacity: self.specs.payload_capacity,
                added_weight: order.weight,
            })
        }
    }

    /// Unload all cargo, clearing manifest & resetting payload
    pub fn unload_all(&mut self) -> Vec<Order> {
        let delivered = self.manifest.drain(..).collect();
        self.current_payload = 0.0;
        self.status = AirplaneStatus::Unloading;
        delivered
    }

    /// Fly directly to `destination_airport`, updating location and fuel. Returns true if successful.
    pub fn fly_to(
        &mut self,
        airport: &Airport,
        airport_coords: &Coordinate,
    ) -> Result<(), GameError> {
        match self.can_fly_to(airport, airport_coords) {
            Err(e) => {
                return Err(e);
            }
            Ok(()) => {
                let distance = self.distance_to(airport_coords);
                let hours = distance / self.specs.cruise_speed;
                let fuel_needed = hours * self.specs.fuel_consumption;
                if fuel_needed <= self.current_fuel {
                    self.current_fuel -= fuel_needed;
                    self.location = Coordinate::new(airport_coords.x, airport_coords.y);
                    self.status = AirplaneStatus::InTransit {
                        destination: Coordinate::new(airport_coords.x, airport_coords.y),
                    };

                    return Ok(());
                } else {
                    return Err(GameError::InsufficientFuel {
                        have: self.current_fuel,
                        need: fuel_needed,
                    });
                }
            }
        };
    }

    /// Refuel to full capacity, switching status to `Refueling`
    pub fn refuel(&mut self) {
        self.current_fuel = self.specs.fuel_capacity;
        self.status = AirplaneStatus::Refueling;
    }
}
