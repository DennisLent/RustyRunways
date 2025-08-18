use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rusty_runways_core::Game;
use serde_json;

#[pyclass]
pub struct PyGame {
    game: Game,
}

#[pymethods]
impl PyGame {
    #[new]
    fn new(seed: Option<u64>, num_airports: Option<usize>, cash: Option<f32>) -> Self {
        PyGame {
            game: Game::new(seed.unwrap_or(0), num_airports, cash.unwrap_or(1_000_000.0)),
        }
    }

    fn reset(&mut self, seed: Option<u64>, num_airports: Option<usize>, cash: Option<f32>) {
        self.game = Game::new(seed.unwrap_or(0), num_airports, cash.unwrap_or(1_000_000.0));
    }

    fn step(&mut self, hours: u64) {
        self.game.advance(hours);
    }

    fn execute(&mut self, cmd: &str) -> PyResult<()> {
        self.game
            .execute_str(cmd)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn state_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.game.observe())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn full_state_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.game).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn load_full_state_json(&mut self, s: &str) -> PyResult<()> {
        self.game = serde_json::from_str(s).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(())
    }

    fn time(&self) -> u64 {
        self.game.time
    }

    fn cash(&self) -> f32 {
        self.game.player.cash
    }
}

#[pymodule]
fn rusty_runways_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyGame>()?;
    Ok(())
}
