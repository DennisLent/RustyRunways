use crate::utils::orders::Order;
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

    /// Generate orders randomly
    /// We assume that larger airports will generate more orders
    pub fn generate_orders(&mut self, seed: u64, num_airports: usize) {
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

        for id in 0..number_orders {
            let order_seed = seed.wrapping_add(self.id as u64).wrapping_add(id as u64);
            self.orders
                .push(Order::new(order_seed, self.id, num_airports));
        }
    }
}
