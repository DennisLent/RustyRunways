use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;
use rusty_runways_core::Game;

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

    fn state_py(&self, py: Python) -> PyResult<PyObject> {
        let s = serde_json::to_string(&self.game.observe())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let json = py.import("json")?;
        json.call_method1("loads", (s,)).map(|o| o.into())
    }

    fn full_state_json(&self) -> PyResult<String> {
        serde_json::to_string(&self.game).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn load_full_state_json(&mut self, s: &str) -> PyResult<()> {
        self.game = serde_json::from_str(s).map_err(|e| PyValueError::new_err(e.to_string()))?;
        self.game.reset_runtime();
        Ok(())
    }

    fn time(&self) -> u64 {
        self.game.time
    }

    fn cash(&self) -> f32 {
        self.game.player.cash
    }

    fn seed(&self) -> u64 {
        self.game.seed()
    }

    fn drain_log(&mut self) -> Vec<String> {
        self.game.drain_log()
    }

    // convenience: expose JSON observation of full state
    fn state_full_json(&self) -> PyResult<String> {
        self.full_state_json()
    }
}

#[pyclass]
pub struct PyVectorEnv {
    envs: Vec<Game>,
    seeds: Vec<u64>,
}

fn parse_arg<T: Clone + for<'a> FromPyObject<'a>>(py: Python<'_>, obj: Option<PyObject>, n: usize, defaults: Vec<T>) -> PyResult<Vec<T>> {
    match obj {
        Some(o) => {
            let any = o.as_ref(py);
            if let Ok(v) = any.extract::<Vec<T>>() {
                if v.len() == n {
                    Ok(v)
                } else if v.len() == 1 {
                    Ok(vec![v[0].clone(); n])
                } else {
                    Err(PyValueError::new_err("length mismatch"))
                }
            } else {
                let val = any.extract::<T>()?;
                Ok(vec![val; n])
            }
        }
        None => Ok(defaults),
    }
}

fn parse_num_airports(py: Python<'_>, obj: Option<PyObject>, n: usize) -> PyResult<Vec<Option<usize>>> {
    match obj {
        Some(o) => {
            let any = o.as_ref(py);
            if let Ok(v) = any.extract::<Vec<usize>>() {
                if v.len() == n {
                    Ok(v.into_iter().map(Some).collect())
                } else if v.len() == 1 {
                    Ok(vec![Some(v[0]); n])
                } else {
                    Err(PyValueError::new_err("length mismatch"))
                }
            } else {
                Ok(vec![Some(any.extract::<usize>()?); n])
            }
        }
        None => Ok(vec![None; n]),
    }
}

#[pymethods]
impl PyVectorEnv {
    #[new]
    fn new(n_envs: usize, seed: Option<u64>, num_airports: Option<usize>, cash: Option<f32>) -> Self {
        let base_seed = seed.unwrap_or(0);
        let mut envs = Vec::with_capacity(n_envs);
        let mut seeds = Vec::with_capacity(n_envs);
        for i in 0..n_envs {
            let s = base_seed + i as u64;
            envs.push(Game::new(s, num_airports, cash.unwrap_or(1_000_000.0)));
            seeds.push(s);
        }
        PyVectorEnv { envs, seeds }
    }

    fn env_count(&self) -> usize {
        self.envs.len()
    }

    fn __len__(&self) -> usize {
        self.envs.len()
    }

    fn seeds(&self) -> Vec<u64> {
        self.seeds.clone()
    }

    fn reset_all(
        &mut self,
        py: Python,
        seed: Option<PyObject>,
        num_airports: Option<PyObject>,
        cash: Option<PyObject>,
    ) -> PyResult<()> {
        let n = self.envs.len();
        let seeds = match seed {
            Some(o) => {
                let any = o.bind(py);
                if let Ok(seq) = any.downcast::<pyo3::types::PyList>() {
                    let v: Vec<u64> = seq.extract()?;
                    if v.len() == n {
                        v
                    } else if v.len() == 1 {
                        (0..n).map(|i| v[0] + i as u64).collect()
                    } else {
                        return Err(PyValueError::new_err("length mismatch"));
                    }
                } else {
                    let base: u64 = any.extract()?;
                    (0..n).map(|i| base + i as u64).collect()
                }
            }
            None => self.seeds.clone(),
        };
        let airports = parse_num_airports(py, num_airports, n)?;
        let cashes = parse_arg(py, cash, n, vec![1_000_000.0; n])?;
        self.seeds = seeds.clone();
        for i in 0..n {
            self.envs[i] = Game::new(seeds[i], airports[i], cashes[i]);
        }
        Ok(())
    }

    fn reset_at(
        &mut self,
        idx: usize,
        seed: Option<u64>,
        num_airports: Option<usize>,
        cash: Option<f32>,
    ) {
        let s = seed.unwrap_or(self.seeds[idx]);
        self.seeds[idx] = s;
        let c = cash.unwrap_or(1_000_000.0);
        self.envs[idx] = Game::new(s, num_airports, c);
    }

    fn step_all(&mut self, py: Python, hours: u64, parallel: Option<bool>) {
        if parallel.unwrap_or(true) {
            py.allow_threads(|| {
                self.envs.par_iter_mut().for_each(|g| g.advance(hours));
            });
        } else {
            for g in &mut self.envs {
                g.advance(hours);
            }
        }
    }

    fn step_masked(&mut self, py: Python, hours: u64, mask: Vec<bool>, parallel: Option<bool>) -> PyResult<()> {
        if mask.len() != self.envs.len() {
            return Err(PyValueError::new_err("mask length mismatch"));
        }
        if parallel.unwrap_or(true) {
            py.allow_threads(|| {
                self.envs
                    .par_iter_mut()
                    .zip(mask.into_par_iter())
                    .for_each(|(g, m)| {
                        if m {
                            g.advance(hours);
                        }
                    });
            });
        } else {
            for (g, m) in self.envs.iter_mut().zip(mask.into_iter()) {
                if m {
                    g.advance(hours);
                }
            }
        }
        Ok(())
    }

    fn execute_all(
        &mut self,
        py: Python,
        cmds: Vec<Option<String>>,
        parallel: Option<bool>,
    ) -> PyResult<Vec<(bool, Option<String>)>> {
        if cmds.len() != self.envs.len() {
            return Err(PyValueError::new_err("commands length mismatch"));
        }
        let n = self.envs.len();
        let mut results = Vec::with_capacity(n);
        if parallel.unwrap_or(true) {
            py.allow_threads(|| {
                for (g, cmd) in self.envs.iter_mut().zip(cmds.into_iter()) {
                    if let Some(c) = cmd {
                        match g.execute_str(&c) {
                            Ok(_) => results.push((true, None)),
                            Err(e) => results.push((false, Some(e.to_string()))),
                        }
                    } else {
                        results.push((true, None));
                    }
                }
            });
        } else {
            for (g, cmd) in self.envs.iter_mut().zip(cmds.into_iter()) {
                if let Some(c) = cmd {
                    match g.execute_str(&c) {
                        Ok(_) => results.push((true, None)),
                        Err(e) => results.push((false, Some(e.to_string()))),
                    }
                } else {
                    results.push((true, None));
                }
            }
        }
        Ok(results)
    }

    fn state_all_json(&self) -> PyResult<Vec<String>> {
        self.envs
            .iter()
            .map(|g| {
                serde_json::to_string(&g.observe())
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .collect()
    }

    fn state_all_py(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let json = py.import("json")?;
        self.envs
            .iter()
            .map(|g| {
                let s = serde_json::to_string(&g.observe())
                    .map_err(|e| PyValueError::new_err(e.to_string()))?;
                json.call_method1("loads", (s,)).map(|o| o.into())
            })
            .collect()
    }

    fn times(&self) -> Vec<u64> {
        self.envs.iter().map(|g| g.time).collect()
    }

    fn cashes(&self) -> Vec<f32> {
        self.envs.iter().map(|g| g.player.cash).collect()
    }

    fn drain_logs(&mut self) -> Vec<Vec<String>> {
        self.envs.iter_mut().map(|g| g.drain_log()).collect()
    }
}

#[pymodule]
fn rusty_runways_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyGame>()?;
    m.add_class::<PyVectorEnv>()?;
    Ok(())
}

