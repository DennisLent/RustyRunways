use crate::utils::{
    airplanes::airplane::Airplane,
    errors::GameError,
    orders::{DemandGenerationParams, Order, order::OrderAirportInfo},
};
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

fn default_base_fuel_price() -> f32 {
    0.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Airport {
    pub id: usize,
    pub name: String,
    pub runway_length: f32, // Limits the types of airplanes that can take off and land
    pub fuel_price: f32,    // price/L
    #[serde(default = "default_base_fuel_price")]
    pub base_fuel_price: f32,
    pub landing_fee: f32, // standard cost that gets multiplied by airplane per ton of mtow
    pub parking_fee: f32, // standard fee per hour
    pub orders: Vec<Order>, // list of current orders
    pub fuel_sold: f32,   // demand based on how much fuel was bought
}

impl Airport {
    /// Helper function to generate unique names for each airport
    fn generate_name(mut id: usize) -> String {
        let mut bytes = [b'A'; 3];
        for i in (0..3).rev() {
            bytes[i] = b'A' + (id % 26) as u8;
            id /= 26;
        }

        String::from_utf8(bytes.to_vec()).unwrap()
    }

    /// Generate an airport using a seed and an id.
    ///
    /// Parameters
    /// - `seed`: RNG seed used for deterministic generation.
    /// - `id`: Airport identifier to assign.
    ///
    /// Returns
    /// - `Airport`: A randomly configured airport with plausible properties.
    pub fn generate_random(seed: u64, id: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(id as u64));

        let name = Airport::generate_name(id);

        // Aiport runways can vary between 245 and 5500 m
        // Runway length will help us determine the other aspects about this airport
        let runway_length: f32 = rng.gen_range(245.0..=5500.0);

        // Can be anywhere between 0.5 and 2.5 per liter
        let fuel_price: f32 = rng.gen_range(0.5..=2.5);

        // From research online I see from 2.4 / ton on small ones to 8 on large ones
        let landing_fee: f32 = match runway_length {
            245.0..500.0 => rng.gen_range(2.4..=3.0),
            500.0..1500.0 => rng.gen_range(3.1..=4.0),
            1500.0..2500.0 => rng.gen_range(4.1..=5.0),
            2500.0..3500.0 => rng.gen_range(5.1..=6.0),
            _ => rng.gen_range(6.1..=9.0),
        };

        // Fee per hour based on the runway length (assume this is linked to the size of the airport)
        let parking_fee = match runway_length {
            245.0..=1000.0 => rng.gen_range(5.0..=15.0),
            1000.0..=3000.0 => rng.gen_range(15.0..=30.0),
            _ => rng.gen_range(30.0..=50.0),
        };

        Airport {
            id,
            name,
            runway_length,
            fuel_price,
            base_fuel_price: fuel_price,
            landing_fee,
            parking_fee,
            orders: Vec::new(),
            fuel_sold: 0.0,
        }
    }

    /// Generate orders randomly.
    ///
    /// Larger airports generate more orders. Passenger orders are created in groups in
    /// addition to cargo orders.
    ///
    /// Parameters
    /// - `seed`: RNG seed to produce reproducible orders.
    /// - `airports`: Metadata of all airports to choose valid destinations from.
    /// - `next_order_id`: Mutable counter; incremented as new orders are created.
    /// - `params`: Demand and tuning parameters.
    pub fn generate_orders(
        &mut self,
        seed: u64,
        airports: &[OrderAirportInfo],
        next_order_id: &mut usize,
        params: &DemandGenerationParams,
    ) {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(self.id as u64));

        let number_orders: usize = match self.runway_length {
            245.0..500.0 => rng.gen_range(2..=4),
            500.0..1500.0 => rng.gen_range(5..=8),
            1500.0..2500.0 => rng.gen_range(9..=15),
            2500.0..3500.0 => rng.gen_range(15..=24),
            _ => rng.gen_range(25..=40),
        };

        let passenger_groups: usize = match self.runway_length {
            245.0..500.0 => rng.gen_range(1..=2),
            500.0..1500.0 => rng.gen_range(2..=4),
            1500.0..2500.0 => rng.gen_range(4..=7),
            2500.0..3500.0 => rng.gen_range(6..=10),
            _ => rng.gen_range(10..=18),
        };

        // Clear all orders within the airport
        self.orders.clear();

        for _ in 0..number_orders {
            let order_id = *next_order_id;
            *next_order_id += 1;

            let order_seed = seed
                .wrapping_add(self.id as u64)
                .wrapping_add(order_id as u64);
            self.orders.push(Order::new_cargo(
                order_seed,
                order_id,
                self.id,
                airports,
                &params.cargo,
            ));
        }

        for _ in 0..passenger_groups {
            let order_id = *next_order_id;
            *next_order_id += 1;

            let order_seed = seed
                .wrapping_add(self.id as u64)
                .wrapping_add(order_id as u64)
                .wrapping_add(13);
            self.orders.push(Order::new_passenger(
                order_seed,
                order_id,
                self.id,
                airports,
                &params.passengers,
            ));
        }

        self.orders.shuffle(&mut rng);
    }

    /// Update order deadlines and remove expired ones.
    ///
    /// Decrements each order's `deadline` by one hour and removes orders with a zero deadline.
    pub fn update_deadline(&mut self) {
        self.orders.retain(|order| order.deadline != 0);

        for order in self.orders.iter_mut() {
            order.deadline -= 1;
        }
    }

    /// Returns the landing fee for a given airplane.
    ///
    /// Parameters
    /// - `airplane`: The airplane landing at this airport.
    ///
    /// Returns
    /// - `f32`: Fee amount based on the airplane MTOW.
    pub fn landing_fee(&self, airplane: &Airplane) -> f32 {
        self.landing_fee * (airplane.specs.mtow / 1000.0)
    }

    /// Returns the fueling fee for a given airplane.
    ///
    /// Parameters
    /// - `airplane`: The airplane to refuel to full.
    ///
    /// Returns
    /// - `f32`: Total cost using current airport fuel price.
    pub fn fueling_fee(&self, airplane: &Airplane) -> f32 {
        self.fuel_price * (airplane.specs.fuel_capacity - airplane.current_fuel)
    }

    /// Record fuel sold to enable dynamic price adjustments later.
    pub fn fuel_supply(&mut self, airplane: &Airplane) {
        let fuel_bought = airplane.specs.fuel_capacity - airplane.current_fuel;
        self.fuel_sold += fuel_bought;
    }

    /// Load a single order into the airplane.
    ///
    /// Parameters
    /// - `order_id`: The order to load from the airport stock.
    /// - `airplane`: Target airplane (must be parked).
    ///
    /// Returns
    /// - `Ok(())` on success.
    /// - `Err(GameError)` if the order is missing or capacity constraints are violated.
    pub fn load_order(
        &mut self,
        order_id: usize,
        airplane: &mut Airplane,
    ) -> Result<(), GameError> {
        // find the position of the order in this airport
        if let Some(pos) = self.orders.iter().position(|o| o.id == order_id) {
            let order = self.orders[pos].clone();
            airplane.validate_payload(&order)?;

            let order = self.orders.remove(pos);
            airplane.load_order(order)?;
            Ok(())
        } else {
            Err(GameError::OrderIdInvalid { id: order_id })
        }
    }

    /// Load multiple orders into the airplane.
    ///
    /// Parameters
    /// - `order_ids`: Order IDs to load in sequence.
    /// - `airplane`: Target airplane.
    ///
    /// Returns
    /// - `Ok(())` if all loads succeed; otherwise the first error encountered.
    pub fn load_orders(
        &mut self,
        order_ids: Vec<usize>,
        airplane: &mut Airplane,
    ) -> Result<(), GameError> {
        for order_id in order_ids.into_iter() {
            match self.load_order(order_id, airplane) {
                Ok(()) => {
                    // Nothing happens, we just keep loading
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Ensure `base_fuel_price` is initialized from the current price.
    pub fn ensure_base_fuel_price(&mut self) {
        if self.base_fuel_price <= 0.0 {
            self.base_fuel_price = self.fuel_price;
        }
    }

    /// Adjust the fuel price based on recent demand (elastic pricing).
    ///
    /// Parameters
    /// - `elasticity`: Price sensitivity factor.
    /// - `min_multiplier`: Floor multiplier relative to base price.
    /// - `max_multiplier`: Cap multiplier relative to base price.
    pub fn adjust_fuel_price(&mut self, elasticity: f32, min_multiplier: f32, max_multiplier: f32) {
        self.ensure_base_fuel_price();

        if self.fuel_sold > 0.0 {
            let increased = self.fuel_price * (1.0 + elasticity);
            self.fuel_price = increased.min(self.base_fuel_price * max_multiplier);
        } else {
            let decreased = self.fuel_price * (1.0 - elasticity);
            self.fuel_price = decreased.max(self.base_fuel_price * min_multiplier);
        }

        // reset usage counter for next interval
        self.fuel_sold = 0.0;
    }
}

// need to run test here because its  private function
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ELASTICITY: f32 = 0.05;

    #[test]
    fn generate_name_check() {
        assert_eq!(&Airport::generate_name(0), "AAA");
        assert_eq!(&Airport::generate_name(1), "AAB");
        assert_eq!(&Airport::generate_name(25), "AAZ");
        assert_eq!(&Airport::generate_name(26), "ABA");
        assert_eq!(&Airport::generate_name(26 * 26 + 26 + 1), "BBB");
    }

    fn sample_airport() -> Airport {
        Airport {
            id: 0,
            name: "AAA".to_string(),
            runway_length: 1000.0,
            fuel_price: 1.5,
            base_fuel_price: 1.5,
            landing_fee: 4.0,
            parking_fee: 10.0,
            orders: Vec::new(),
            fuel_sold: 0.0,
        }
    }

    #[test]
    fn adjust_fuel_price_increases_when_fuel_sold() {
        let mut airport = sample_airport();
        airport.fuel_sold = 500.0;
        airport.adjust_fuel_price(TEST_ELASTICITY, 0.5, 1.5);
        assert!((airport.fuel_price - 1.5 * (1.0 + TEST_ELASTICITY)).abs() < f32::EPSILON);
    }

    #[test]
    fn adjust_fuel_price_decreases_when_idle() {
        let mut airport = sample_airport();
        airport.adjust_fuel_price(TEST_ELASTICITY, 0.5, 1.5);
        assert!((airport.fuel_price - 1.5 * (1.0 - TEST_ELASTICITY)).abs() < f32::EPSILON);
    }

    #[test]
    fn adjust_fuel_price_respects_floor() {
        let mut airport = sample_airport();
        airport.base_fuel_price = 1.5;
        airport.fuel_price = airport.base_fuel_price * 0.3;
        airport.adjust_fuel_price(TEST_ELASTICITY, 0.5, 1.5);
        assert!((airport.fuel_price - airport.base_fuel_price * 0.5).abs() < f32::EPSILON);
    }
}
