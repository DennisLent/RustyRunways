use crate::utils::{
    airplanes::airplane::Airplane, coordinate::Coordinate, errors::GameError, orders::Order,
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Airport {
    pub id: usize,
    pub name: String,
    pub runway_length: f32, // Limits the types of airplanes that can take off and land
    pub fuel_price: f32,    // price/L
    pub landing_fee: f32,   // standard cost that gets multiplied by airplane per ton of mtow
    pub parking_fee: f32,   // standard fee per hour
    pub orders: Vec<Order>, // list of current orders
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

    /// Generate an airport using a seed and an id
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
            landing_fee,
            parking_fee,
            orders: Vec::new(),
        }
    }

    /// Generate orders randomly.
    /// We assume that larger airports will generate more orders.
    pub fn generate_orders(
        &mut self,
        seed: u64,
        airport_coordinates: &[Coordinate],
        num_airports: usize,
        next_order_id: &mut usize,
    ) {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(self.id as u64));

        let number_orders: usize = match self.runway_length {
            245.0..500.0 => rng.gen_range(2..=4),
            500.0..1500.0 => rng.gen_range(5..=8),
            1500.0..2500.0 => rng.gen_range(9..=15),
            2500.0..3500.0 => rng.gen_range(15..=24),
            _ => rng.gen_range(25..=40),
        };

        // Clear all orders within the airport
        self.orders.clear();

        for _ in 0..number_orders {
            let order_id = *next_order_id;
            *next_order_id += 1;

            let order_seed = seed
                .wrapping_add(self.id as u64)
                .wrapping_add(order_id as u64);
            self.orders.push(Order::new(
                order_seed,
                order_id,
                self.id,
                airport_coordinates,
                num_airports,
            ));
        }
    }

    /// Check if any orders have expired, if so we remove them.
    /// Update the deadline hour for each order.
    pub fn update_deadline(&mut self) {
        self.orders.retain(|order| order.deadline != 0);

        for order in self.orders.iter_mut() {
            order.deadline -= 1;
        }
    }

    /// Returns the landing fee for a given airplane.
    pub fn landing_fee(&self, airplane: &Airplane) -> f32 {
        self.landing_fee * (airplane.specs.mtow / 1000.0)
    }

    /// Returns the fueling fee for a given airplane.
    pub fn fueling_fee(&self, airplane: &Airplane) -> f32 {
        self.fuel_price * (airplane.specs.fuel_capacity - airplane.current_fuel)
    }

    /// Load a single order into the airplane
    pub fn load_order(
        &mut self,
        order_id: usize,
        airplane: &mut Airplane,
    ) -> Result<(), GameError> {
        // find the position of the order in this airport
        if let Some(pos) = self.orders.iter().position(|o| o.id == order_id) {
            let order = self.orders[pos].clone();

            // check payload capacity before removing
            if airplane.current_payload + order.weight > airplane.specs.payload_capacity {
                return Err(GameError::MaxPayloadReached {
                    current_capacity: airplane.current_payload,
                    maximum_capacity: airplane.specs.payload_capacity,
                    added_weight: order.weight,
                });
            }

            // remove from airport and load into airplane
            let order = self.orders.remove(pos);
            airplane.load_order(order)?;
            Ok(())
        } else {
            Err(GameError::OrderIdInvalid { id: order_id })
        }
    }

    /// Load multiple orders into the plane
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
}

// need to run test here because its  private function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_name_check() {
        assert_eq!(&Airport::generate_name(0), "AAA");
        assert_eq!(&Airport::generate_name(1), "AAB");
        assert_eq!(&Airport::generate_name(25), "AAZ");
        assert_eq!(&Airport::generate_name(26), "ABA");
        assert_eq!(&Airport::generate_name(26 * 26 + 26 + 1), "BBB");
    }
}
