use serde::{Deserialize, Serialize};
use crate::utils::{coordinate::Coordinate, orders::Order};
use super::models::{AirplaneModel, AirplaneSpecs, AirplaneStatus};

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
    pub fn distance_to(&self, target_coordinates: Coordinate) -> f32 {
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

    /// Load an order if it fits; returns Err(order) if too heavy
    pub fn load_order(&mut self, order: Order) -> Result<(), Order> {
        if self.current_payload + order.weight <= self.specs.payload_capacity {
            self.current_payload += order.weight;
            self.manifest.push(order);
            self.status = AirplaneStatus::Loading;
            Ok(())
        } else {
            Err(order)
        }
    }

    /// Unload all cargo, clearing manifest & resetting payload
    pub fn unload_all(&mut self) -> Vec<Order> {
        let delivered = self.manifest.drain(..).collect();
        self.current_payload = 0.0;
        self.status = AirplaneStatus::Unloading;
        delivered
    }

    /// Fly directly to `dest_coords`, updating location and fuel. Returns true if successful.
    pub fn fly_to(&mut self, dest_coords: Coordinate) -> bool {
        let distance = self.distance_to(dest_coords);
        let hours = distance / self.specs.cruise_speed;
        let fuel_needed = hours * self.specs.fuel_consumption;
        if fuel_needed <= self.current_fuel {
            self.current_fuel -= fuel_needed;
            self.location = dest_coords;
            self.status = AirplaneStatus::InTransit { destination: dest_coords };
            true
        } else {
            false
        }
    }

    /// Refuel to full capacity, switching status to `Refueling`
    pub fn refuel(&mut self) {
        self.current_fuel = self.specs.fuel_capacity;
        self.status = AirplaneStatus::Refueling;
    }
}
