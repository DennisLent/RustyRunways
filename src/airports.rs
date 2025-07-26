use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Airport {
    pub name: String,
    pub runway_length: u32, // To ensure not all airplanes can take off from here
    pub fuel_price: f32, // Fuel price changes
    pub landing_fee: f32, // Typical at airports to have a landing fee (based on MTOW)
    pub parking_fee: f32 // based on amount of time airplane is parked
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub weight: f32,
    pub value: f32, // If it arrives on time
    pub deadline: u32, // in days
    pub origin_id: usize,
    pub destination_id: usize
}