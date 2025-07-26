use super::airplanes::Airplane;

#[derive(Debug, Clone)]
pub struct BasicAirplane {
    pub model: String,
    pub mtow: u32,
    pub fuel_capacity: u32,
    pub current_fuel: f32,
    pub consumption: f32,
    pub speed: f32,
    pub cargo_capacity: u32,
    pub current_cargo_weight: u32,
    pub cargo_manifest: Vec<usize>,
}

impl BasicAirplane {
    /// Heavy but slow hauler: High capacity, lower speed.
    pub fn new_heavy() -> Self {
        BasicAirplane {
            model: "BigHauler 747X".into(),
            mtow: 300000,
            fuel_capacity: 300000,
            current_fuel: 300000.0,
            consumption: 5.0,
            speed: 500.0,
            cargo_capacity: 100000,
            current_cargo_weight: 0,
            cargo_manifest: Vec::new(),
        }
    }
    /// Fast but small: Lower capacity, higher speed.
    pub fn new_speedy() -> Self {
        BasicAirplane {
            model: "SkySwift S21".into(),
            mtow: 50000,
            fuel_capacity: 50000,
            current_fuel: 50000.0,
            consumption: 2.0,
            speed: 800.0,
            cargo_capacity: 10000,
            current_cargo_weight: 0,
            cargo_manifest: Vec::new(),
        }
    }
    /// Jack-of-all-trades: balanced stats.
    pub fn new_jack() -> Self {
        BasicAirplane {
            model: "CloudRunner C50".into(),
            mtow: 150000,
            fuel_capacity: 150000,
            current_fuel: 150000.0,
            consumption: 3.5,
            speed: 650.0,
            cargo_capacity: 50000,
            current_cargo_weight: 0,
            cargo_manifest: Vec::new(),
        }
    }
}

impl Airplane for BasicAirplane {
    fn mtow(&self) -> u32 {
        return self.mtow;
    }

    fn fuel_level(&self) -> f32 {
        return self.current_fuel;
    }

    fn fuel_capacity(&self) -> u32 {
        return self.fuel_capacity;
    }

    fn fuel_consumption(&self) -> f32 {
        return self.consumption;
    }

    fn speed(&self) -> f32 {
        return self.speed;
    }

    fn cargo_load(&self) -> u32 {
        return self.current_cargo_weight;
    }

    fn cargo_capacity(&self) -> u32 {
        return self.cargo_capacity;
    }

    fn min_runway(&self) -> u32 {
        return (self.mtow as f32 * 0.01) as u32;
    }

    fn max_distance(&self) -> u32 {
        return (self.fuel_capacity as f32 / self.consumption) as u32
    }

    fn fly(&mut self, distance: f32) -> Result<f32, String> {
        let fuel_required = distance * self.consumption;
        if self.current_fuel < fuel_required {
            return Err("Not enough fuel".into())
        }
        self.current_fuel -= fuel_required;
        Ok(distance / self.speed)
    }

    fn refuel(&mut self) -> f32 {
        let added = self.fuel_capacity as f32 - self.current_fuel;
        self.current_fuel = self.fuel_capacity as f32;
        return added;
    }

    fn load_cargo(&mut self, order_id: usize, weight: f32) -> Result<(), String> {
        let w = weight as u32;
        if self.current_cargo_weight + w > self.cargo_capacity {
            Err("Capacity exceeded".into())
        } else{
            self.current_cargo_weight += w;
            self.cargo_manifest.push(order_id);
            Ok(())
        }
    }

    fn unload_cargo(&mut self) -> Vec<usize> {
        let manifest = self.cargo_manifest.clone();
        self.cargo_manifest.clear();
        self.current_cargo_weight = 0;
        return manifest;
    }

    fn set_fuel(&mut self, liters: f32) {
        self.current_fuel = liters;
    }
}
