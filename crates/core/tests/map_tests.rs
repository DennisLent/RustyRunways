use rusty_runways_core::utils::map::Map;

#[test]
fn map_generation_is_deterministic() {
    let seed = 42;
    let map1 = Map::generate_from_seed(seed, Some(5));
    let map2 = Map::generate_from_seed(seed, Some(5));
    let json1 = serde_json::to_string(&map1).unwrap();
    let json2 = serde_json::to_string(&map2).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn clustered_generation_produces_local_neighbors() {
    let map = Map::generate_from_seed(7, Some(12));
    for (airport, coord) in &map.airports {
        let mut min_distance = f32::INFINITY;
        for (other_airport, other_coord) in &map.airports {
            if airport.id == other_airport.id {
                continue;
            }
            let dx = coord.x - other_coord.x;
            let dy = coord.y - other_coord.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < min_distance {
                min_distance = distance;
            }
        }
        assert!(
            min_distance < 2_500.0,
            "expected at least one neighbor within 2500km for airport {}",
            airport.id
        );
    }
}
