pub enum Action {
    Fly { destination_id: usize },
    Refuel,
    Load { order_id: usize },
    Unload,
    Wait,
    Maintenance
}

pub trait Airplane {
    /// Maximum takeoff weight (kg).
    fn mtow(&self) -> u32;
    /// Current fuel level (liters).
    fn fuel_level(&self) -> f32;
    /// Maximum fuel capacity (liters).
    fn fuel_capacity(&self) -> u32;
    /// Fuel consumption (liters per km).
    fn fuel_consumption(&self) -> f32;
    /// Cruising speed (km/h).
    fn speed(&self) -> f32;
    /// Current cargo load (kg).
    fn cargo_load(&self) -> u32;
    /// Maximum cargo capacity (kg).
    fn cargo_capacity(&self) -> u32;
    /// Minimum runway length required (m).
    fn min_runway(&self) -> u32;
    /// Maximum flight distance on full tank (km).
    fn max_distance(&self) -> u32;

    /// Attempt flight: consumes fuel and returns time taken (hours).
    fn fly(&mut self, distance: f32) -> Result<f32, String>;
    /// Refuel to full capacity; returns liters added.
    fn refuel(&mut self) -> f32;
    /// Load weight; Err if exceeds capacity.
    fn load_cargo(&mut self, order_id: usize, weight: f32) -> Result<(), String>;
    /// Unload all cargo; returns unloaded weight.
    fn unload_cargo(&mut self) -> Vec<usize>;
    /// Set current fuel level.
    fn set_fuel(&mut self, liters: f32);
}