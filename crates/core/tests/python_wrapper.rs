use rusty_runways_core::Game;

#[test]
fn deterministic_replay() {
    let mut g = Game::new(1, Some(5), 1_000_000.0);
    g.step(1);
    let obs = serde_json::to_string(&g.observe()).unwrap();
    // hash observation to ensure determinism
    let hash = blake3::hash(obs.as_bytes()).to_hex().to_string();
    assert_eq!(
        hash,
        "3871b9c3d15a4a2ac5819999ba610c9f61d7773143028b71566d70807f90742f"
    );
}

#[test]
fn parser_passthrough() {
    let mut g1 = Game::new(1, Some(5), 1_000_000.0);
    g1.step(1);
    let mut g2 = Game::new(1, Some(5), 1_000_000.0);
    g2.execute_str("ADVANCE 1").unwrap();
    let o1 = serde_json::to_string(&g1.observe()).unwrap();
    let o2 = serde_json::to_string(&g2.observe()).unwrap();
    assert_eq!(o1, o2);
}

#[test]
fn serialization_roundtrip() {
    let mut g = Game::new(1, Some(5), 1_000_000.0);
    g.step(1);
    let dump = serde_json::to_string(&g).unwrap();
    let h: Game = serde_json::from_str(&dump).unwrap();
    assert_eq!(g.time, h.time);
    assert_eq!(g.player.cash, h.player.cash);
    assert_eq!(g.airplanes.len(), h.airplanes.len());
}

#[test]
fn concurrency_sanity() {
    use std::thread;
    let mut games: Vec<Game> = (0..4)
        .map(|i| Game::new(i as u64, Some(5), 1_000_000.0))
        .collect();
    thread::scope(|s| {
        for g in &mut games {
            s.spawn(move || {
                g.step(1);
            });
        }
    });
}
