mod utils;

use utils::map::Map;

fn main() {
    let mut map = Map::generate_from_seed(1, Some(4));
    println!("map: {:?}", map);

    map.restock_airports();
    println!("map: {:?}", map);

}
