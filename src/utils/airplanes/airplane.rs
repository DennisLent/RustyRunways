use serde::{Deserialize, Serialize};
use crate::utils::orders::Order;
use super::models::{AirplaneModel, AirplaneSpecs, AirplaneStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airplane {
    pub id: usize,
    pub model: AirplaneModel,
    pub specs: AirplaneSpecs,
    pub status: AirplaneStatus,
    pub location: (usize, usize),
    pub current_fuel: f32,
    pub current_payload: f32,
    pub manifest: Vec<Order>
}

impl Airplane {

    pub fn new(id: usize, model: AirplaneModel, home_airport_coordinates: (usize, usize)) -> Self {
        let specs = model.specs();

        Airplane { 
            id, 
            model, 
            specs, 
            status: AirplaneStatus::Parked,
            location: (home_airport_coordinates.0, home_airport_coordinates.1),
            current_fuel: specs.fuel_capacity,
            current_payload: 0.0,
            manifest: Vec::new() }
    }

    pub fn distance_to(&self, target_coordinates: (usize, usize)) -> f32 {
        let dx = (self.location.0 - target_coordinates.0) as f32;
        let dy = (self.location.1 - target_coordinates.1) as f32;

        (dx*dx + dy*dy).sqrt()
    }

    pub fn endurance_hours(&self) -> f32 {
        self.current_fuel / self.specs.fuel_consumption
    }

    pub fn max_range(&self) -> f32 {
        self.endurance_hours() * self.specs.cruise_speed
    }

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


    pub fn unload_all(&mut self) -> Vec<Order> {
        let delivered = self.manifest.drain(..).collect();
        self.current_payload = 0.0;
        self.status = AirplaneStatus::Unloading;
        delivered
    }

    pub fn fly_to(&mut self, dest_coords: (usize, usize)) -> bool {
        let distance = self.distance_to(dest_coords);
        let hours = distance / self.specs.cruise_speed;
        let fuel_needed = hours * self.specs.fuel_consumption;
        if fuel_needed <= self.current_fuel {
            self.current_fuel -= fuel_needed;
            self.location = (dest_coords.0, dest_coords.1);
            self.status = AirplaneStatus::InTransit;
            true
        } else {
            false
        }
    }

    pub fn refuel(&mut self) {
        self.current_fuel = self.specs.fuel_capacity;
        self.status = AirplaneStatus::Refueling;
    }
}
