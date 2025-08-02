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
