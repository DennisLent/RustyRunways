use super::models::{AirplaneModel, AirplaneSpecs, AirplaneStatus};
use crate::{
    events::GameTime,
    utils::{
        airport::Airport,
        coordinate::Coordinate,
        errors::GameError,
        orders::{Order, order::OrderPayload},
    },
};
use serde::{Deserialize, Serialize};

const LAMBDA0: f32 = 0.005;
const K: f32 = 0.01;

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
    #[serde(default)]
    pub current_passengers: u32,
    pub manifest: Vec<Order>,
    pub hours_since_maintenance: GameTime,
    pub needs_maintenance: bool,
}

impl Airplane {
    /// Create a fresh airplane, parked and fueled up at `home_airport_coordinates`.
    ///
    /// Parameters
    /// - `id`: Unique plane identifier.
    /// - `model`: Airplane model enum.
    /// - `home_airport_coordinates`: Initial location.
    ///
    /// Returns
    /// - `Airplane`: New plane in `Parked` status with full fuel.
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
            current_passengers: 0,
            manifest: Vec::new(),
            hours_since_maintenance: 0,
            needs_maintenance: false,
        }
    }

    /// Euclidean distance from current location to `target_coordinates`.
    pub fn distance_to(&self, target_coordinates: &Coordinate) -> f32 {
        let dx = self.location.x - target_coordinates.x;
        let dy = self.location.y - target_coordinates.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// How many hours can we fly on current fuel?
    pub fn endurance_hours(&self) -> f32 {
        self.current_fuel / self.specs.fuel_consumption
    }

    /// Maximum range (km) before refuel.
    pub fn max_range(&self) -> f32 {
        self.endurance_hours() * self.specs.cruise_speed
    }

    /// Check reachability and landing ability for a target airport.
    ///
    /// Returns `Ok(())` if within range and runway length is sufficient.
    pub fn can_fly_to(
        &self,
        airport: &Airport,
        airport_coords: &Coordinate,
    ) -> Result<(), GameError> {
        // Cannot fly to the airport we are currently at
        if airport_coords == &self.location {
            return Err(GameError::SameAirport);
        }

        let distance = self.distance_to(airport_coords);

        // Cannot go this far
        if distance > self.max_range() {
            return Err(GameError::OutOfRange {
                distance,
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

        Ok(())
    }

    /// Load an order if it fits; returns error if too heavy or incompatible.
    pub fn load_order(&mut self, order: Order) -> Result<(), GameError> {
        self.validate_payload(&order)?;

        match order.payload {
            OrderPayload::Cargo { weight, .. } => {
                self.current_payload += weight;
            }
            OrderPayload::Passengers { count } => {
                self.current_passengers = self.current_passengers.saturating_add(count);
            }
        }

        self.manifest.push(order);
        self.status = AirplaneStatus::Loading;
        Ok(())
    }

    /// Unload all cargo and passengers, clearing the manifest and resetting payload counters.
    pub fn unload_all(&mut self) -> Vec<Order> {
        let delivered = self.manifest.drain(..).collect();
        self.current_payload = 0.0;
        self.current_passengers = 0;
        self.status = AirplaneStatus::Unloading;
        delivered
    }

    /// Unload a specific order by ID, updating current load counters accordingly.
    pub fn unload_order(&mut self, order_id: usize) -> Result<Order, GameError> {
        if let Some(idx) = self.manifest.iter().position(|order| order.id == order_id) {
            let delivered = self.manifest.remove(idx);

            match &delivered.payload {
                OrderPayload::Cargo { weight, .. } => {
                    // Ensure we don't have some FLOP rounding errors
                    self.current_payload = (self.current_payload - weight).max(0.0);
                }
                OrderPayload::Passengers { count } => {
                    self.current_passengers = self.current_passengers.saturating_sub(*count);
                }
            }
            self.status = AirplaneStatus::Unloading;

            Ok(delivered)
        } else {
            Err(GameError::OrderIdInvalid { id: order_id })
        }
    }

    /// Validate whether an order can be loaded given current load and capacities.
    pub fn validate_payload(&self, order: &Order) -> Result<(), GameError> {
        match &order.payload {
            OrderPayload::Cargo { weight, .. } => {
                if self.specs.payload_capacity <= 0.0 {
                    return Err(GameError::PayloadTypeUnsupported {
                        plane_model: format!("{:?}", self.model),
                        payload: "cargo".into(),
                    });
                }
                if self.current_payload + weight > self.specs.payload_capacity + f32::EPSILON {
                    return Err(GameError::MaxPayloadReached {
                        current_capacity: self.current_payload,
                        maximum_capacity: self.specs.payload_capacity,
                        added_weight: *weight,
                    });
                }
            }
            OrderPayload::Passengers { count } => {
                if self.specs.passenger_capacity == 0 {
                    return Err(GameError::PayloadTypeUnsupported {
                        plane_model: format!("{:?}", self.model),
                        payload: "passengers".into(),
                    });
                }
                let next = self.current_passengers.saturating_add(*count);
                if next > self.specs.passenger_capacity {
                    return Err(GameError::PassengerCapacityReached {
                        current_capacity: self.current_passengers,
                        maximum_capacity: self.specs.passenger_capacity,
                        added_passengers: *count,
                    });
                }
            }
        }
        Ok(())
    }

    //// Check runway & fuel, consume fuel, and return flight time in hours.
    pub fn consume_flight_fuel(
        &mut self,
        airport: &Airport,
        airport_coords: &Coordinate,
    ) -> Result<GameTime, GameError> {
        // runway & range check
        self.can_fly_to(airport, airport_coords)?;

        // distance & fuel
        let dist = self.distance_to(airport_coords);
        let hours_f = dist / self.specs.cruise_speed;
        let fuel_needed = hours_f * self.specs.fuel_consumption;
        if fuel_needed > self.current_fuel {
            return Err(GameError::InsufficientFuel {
                have: self.current_fuel,
                need: fuel_needed,
            });
        }
        // burn the fuel
        self.current_fuel -= fuel_needed;
        Ok(hours_f.ceil() as GameTime)
    }

    /// Refuel to full capacity, switching status to `Refueling`.
    pub fn refuel(&mut self) {
        self.current_fuel = self.specs.fuel_capacity;
        self.status = AirplaneStatus::Refueling;
    }

    /// Perform maintenance, resetting the maintenance counter and switching to `Maintenance` status.
    pub fn maintenance(&mut self) {
        self.hours_since_maintenance = 0;
        self.status = AirplaneStatus::Maintenance;
    }

    pub fn add_hours_since_maintenance(&mut self) {
        self.hours_since_maintenance += 1;
    }

    // Check the risk of failure based on the amount of hours since last maintenance
    pub fn risk_of_failure(&self) -> f32 {
        let lambda: f32 = LAMBDA0 * (K * self.hours_since_maintenance as f32).exp();

        1.0 - (-lambda).exp()
    }
}
