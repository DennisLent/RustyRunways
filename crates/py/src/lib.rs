use pyo3::prelude::*;
use rayon::prelude::*;
use rusty_runways_core::Game;

#[pyclass]
struct PyGame {
    game: Game,
}

#[pymethods]
impl PyGame {
    #[new]
    fn new(seed: Option<u64>, num_airports: Option<usize>, cash: Option<f32>) -> PyResult<Self> {
        let seed = seed.unwrap_or(0);
        let cash = cash.unwrap_or(1_000_000.0);
        Ok(PyGame {
            game: Game::new(seed, num_airports, cash),
        })
    }

    fn reset(
        &mut self,
        seed: Option<u64>,
        num_airports: Option<usize>,
        cash: Option<f32>,
    ) -> PyResult<()> {
        let seed = seed.unwrap_or(self.game.map.seed);
        let cash = cash.unwrap_or(self.game.player.cash);
        *self = PyGame {
            game: Game::new(seed, num_airports, cash),
        };
        Ok(())
    }

    fn seed(&self) -> PyResult<u64> {
        Ok(self.game.map.seed)
    }

    fn step(&mut self, hours: u64) -> PyResult<()> {
        self.game.step(hours);
        Ok(())
    }

    fn execute(&mut self, cmd: &str) -> PyResult<()> {
        self.game
            .execute_str(cmd)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn state_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.game.observe())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn full_state_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.game)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn load_full_state_json(&mut self, s: &str) -> PyResult<()> {
        self.game = serde_json::from_str(s)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(())
    }

    fn drain_log(&mut self) -> PyResult<Vec<String>> {
        Ok(self.game.drain_log())
    }

    fn time(&self) -> PyResult<u64> {
        Ok(self.game.time)
    }

    fn cash(&self) -> PyResult<f32> {
        Ok(self.game.player.cash)
    }
}

#[pyclass]
struct PyVectorEnv {
    envs: Vec<Game>,
}

#[pymethods]
impl PyVectorEnv {
    #[new]
    fn new(
        n_envs: usize,
        seed: Option<u64>,
        num_airports: Option<usize>,
        cash: Option<f32>,
    ) -> PyResult<Self> {
        let base_seed = seed.unwrap_or(0);
        let cash = cash.unwrap_or(1_000_000.0);
        let mut envs = Vec::with_capacity(n_envs);
        for i in 0..n_envs {
            envs.push(Game::new(base_seed + i as u64, num_airports, cash));
        }
        Ok(PyVectorEnv { envs })
    }

    fn reset_all(
        &mut self,
        seed: Option<u64>,
        num_airports: Option<usize>,
        cash: Option<f32>,
    ) -> PyResult<()> {
        let base_seed = seed.unwrap_or(0);
        let cash = cash.unwrap_or(1_000_000.0);
        for (i, g) in self.envs.iter_mut().enumerate() {
            *g = Game::new(base_seed + i as u64, num_airports, cash);
        }
        Ok(())
    }

    fn step_all(&mut self, hours: u64, parallel: Option<bool>) -> PyResult<()> {
        if parallel.unwrap_or(false) {
            Python::with_gil(|py| {
                py.allow_threads(|| {
                    self.envs.par_iter_mut().for_each(|g| g.step(hours));
                });
            });
        } else {
            for g in &mut self.envs {
                g.step(hours);
            }
        }
        Ok(())
    }

    fn execute_all(
        &mut self,
        cmds: Vec<String>,
        parallel: Option<bool>,
    ) -> PyResult<Vec<Option<String>>> {
        if cmds.len() != self.envs.len() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "cmds length mismatch",
            ));
        }
        let mut results: Vec<Option<String>> = Vec::with_capacity(self.envs.len());
        if parallel.unwrap_or(false) {
            Python::with_gil(|py| {
                py.allow_threads(|| {
                    results = self
                        .envs
                        .par_iter_mut()
                        .zip(cmds.par_iter())
                        .map(|(g, c)| g.execute_str(c).err().map(|e| e.to_string()))
                        .collect();
                });
            });
        } else {
            results = self
                .envs
                .iter_mut()
                .zip(cmds)
                .map(|(g, c)| g.execute_str(&c).err().map(|e| e.to_string()))
                .collect();
        }
        Ok(results)
    }

    fn states_json(&self) -> PyResult<Vec<String>> {
        self.envs
            .iter()
            .map(|g| {
                serde_json::to_string(&g.observe())
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
            })
            .collect()
    }

    fn drain_logs(&mut self) -> PyResult<Vec<Vec<String>>> {
        Ok(self.envs.iter_mut().map(|g| g.drain_log()).collect())
    }
}

#[pymodule]
fn rusty_runways_py(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGame>()?;
    m.add_class::<PyVectorEnv>()?;
    Ok(())
}
