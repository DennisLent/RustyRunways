use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;
use rusty_runways_core::Game;
use rusty_runways_core::config::WorldConfig;

#[pyclass]
pub struct GameEnv {
    game: Game,
}

#[pymethods]
impl GameEnv {
    #[new]
    #[pyo3(signature = (seed=None, num_airports=None, cash=None, config_path=None))]
    #[pyo3(text_signature = "(/, seed=None, num_airports=None, cash=None, config_path=None)")]
    fn new(
        seed: Option<u64>,
        num_airports: Option<usize>,
        cash: Option<f32>,
        config_path: Option<String>,
    ) -> PyResult<Self> {
        if let Some(path) = config_path {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| PyValueError::new_err(format!("read {}: {}", path, e)))?;
            let cfg: WorldConfig = serde_yaml::from_str(&text)
                .map_err(|e| PyValueError::new_err(format!("yaml: {}", e)))?;
            let game = Game::from_config(cfg).map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(GameEnv { game });
        }
        Ok(GameEnv {
            game: Game::new(seed.unwrap_or(0), num_airports, cash.unwrap_or(650_000.0)),
        })
    }

    #[pyo3(signature = (seed=None, num_airports=None, cash=None, config_path=None))]
    #[pyo3(text_signature = "(/, seed=None, num_airports=None, cash=None, config_path=None)")]
    fn reset(
        &mut self,
        seed: Option<u64>,
        num_airports: Option<usize>,
        cash: Option<f32>,
        config_path: Option<String>,
    ) -> PyResult<()> {
        if let Some(path) = config_path {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| PyValueError::new_err(format!("read {}: {}", path, e)))?;
            let cfg: WorldConfig = serde_yaml::from_str(&text)
                .map_err(|e| PyValueError::new_err(format!("yaml: {}", e)))?;
            self.game = Game::from_config(cfg).map_err(|e| PyValueError::new_err(e.to_string()))?;
            return Ok(());
        }
        self.game = Game::new(seed.unwrap_or(0), num_airports, cash.unwrap_or(650_000.0));
        Ok(())
    }

    fn step(&mut self, hours: u64) {
        self.game.advance(hours);
    }

    fn execute(&mut self, cmd: &str) -> PyResult<()> {
        self.game
            .execute_str(cmd)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[pyo3(text_signature = "(plane_id)")]
    fn sell_plane(&mut self, plane_id: usize) -> PyResult<f32> {
        self.game
            .sell_plane(plane_id)
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

    fn models_json(&self) -> PyResult<String> {
        #[derive(serde::Serialize)]
        struct ModelDto {
            name: String,
            mtow: f32,
            cruise_speed: f32,
            fuel_capacity: f32,
            fuel_consumption: f32,
            operating_cost: f32,
            payload_capacity: f32,
            passenger_capacity: u32,
            purchase_price: f32,
            min_runway_length: f32,
            role: String,
        }
        let models: Vec<ModelDto> = self
            .game
            .available_models()
            .into_iter()
            .map(|(name, s)| ModelDto {
                name,
                mtow: s.mtow,
                cruise_speed: s.cruise_speed,
                fuel_capacity: s.fuel_capacity,
                fuel_consumption: s.fuel_consumption,
                operating_cost: s.operating_cost,
                payload_capacity: s.payload_capacity,
                passenger_capacity: s.passenger_capacity,
                purchase_price: s.purchase_price,
                min_runway_length: s.min_runway_length,
                role: format!("{:?}", s.role),
            })
            .collect();
        serde_json::to_string(&models).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn models_py(&self, py: Python) -> PyResult<PyObject> {
        let s = self.models_json()?;
        let json = py.import("json")?;
        json.call_method1("loads", (s,)).map(|o| o.into())
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

    /// Return order IDs at the airport where the given plane is currently located.
    ///
    /// Parameters
    /// ----------
    /// plane_id : usize
    ///     Game plane identifier.
    ///
    /// Returns
    /// -------
    /// Vec<usize>
    ///     List of order IDs available at that airport (empty if none).
    #[pyo3(text_signature = "(plane_id)")]
    fn orders_at_plane(&self, plane_id: usize) -> Vec<usize> {
        // find plane by id
        if let Some(p) = self.game.airplanes.iter().find(|p| p.id == plane_id) {
            let loc = p.location;
            if let Some((ap, _)) = self.game.map.airports.iter().find(|(_, c)| *c == loc) {
                return ap.orders.iter().map(|o| o.id).collect();
            }
        }
        Vec::new()
    }

    /// Return all airport IDs in the current world.
    ///
    /// Returns
    /// -------
    /// Vec<usize>
    ///     List of airport identifiers.
    #[pyo3(text_signature = "()")]
    fn airport_ids(&self) -> Vec<usize> {
        self.game.map.airports.iter().map(|(a, _)| a.id).collect()
    }
}

#[pyclass]
pub struct VectorGameEnv {
    envs: Vec<Game>,
    seeds: Vec<u64>,
}

fn parse_arg<T: Clone + for<'a> FromPyObject<'a>>(
    py: Python<'_>,
    obj: Option<PyObject>,
    n: usize,
    defaults: Vec<T>,
) -> PyResult<Vec<T>> {
    match obj {
        Some(o) => {
            let any = o.as_ref();
            if let Ok(v) = any.extract::<Vec<T>>(py) {
                if v.len() == n {
                    Ok(v)
                } else if v.len() == 1 {
                    Ok(vec![v[0].clone(); n])
                } else {
                    Err(PyValueError::new_err("length mismatch"))
                }
            } else {
                let val = any.extract::<T>(py)?;
                Ok(vec![val; n])
            }
        }
        None => Ok(defaults),
    }
}

fn parse_num_airports(
    py: Python<'_>,
    obj: Option<PyObject>,
    n: usize,
) -> PyResult<Vec<Option<usize>>> {
    match obj {
        Some(o) => {
            let any = o.as_ref();
            if let Ok(v) = any.extract::<Vec<usize>>(py) {
                if v.len() == n {
                    Ok(v.into_iter().map(Some).collect())
                } else if v.len() == 1 {
                    Ok(vec![Some(v[0]); n])
                } else {
                    Err(PyValueError::new_err("length mismatch"))
                }
            } else {
                Ok(vec![Some(any.extract::<usize>(py)?); n])
            }
        }
        None => Ok(vec![None; n]),
    }
}

#[pymethods]
impl VectorGameEnv {
    #[new]
    #[pyo3(signature = (n_envs, seed=None, num_airports=None, cash=None, config_path=None))]
    fn new(
        n_envs: usize,
        seed: Option<u64>,
        num_airports: Option<usize>,
        cash: Option<f32>,
        config_path: Option<String>,
    ) -> Self {
        let base_seed = seed.unwrap_or(0);
        let mut envs = Vec::with_capacity(n_envs);
        let mut seeds = Vec::with_capacity(n_envs);
        let paths: Vec<Option<String>> = vec![config_path; n_envs];
        for (i, p_opt) in paths.iter().enumerate() {
            if let Some(p) = p_opt {
                if let Ok(text) = std::fs::read_to_string(p) {
                    if let Ok(cfg) = serde_yaml::from_str::<WorldConfig>(&text) {
                        if let Ok(g) = Game::from_config(cfg) {
                            seeds.push(g.seed());
                            envs.push(g);
                            continue;
                        }
                    }
                }
            }
            let s = base_seed + i as u64;
            envs.push(Game::new(s, num_airports, cash.unwrap_or(650_000.0)));
            seeds.push(s);
        }
        VectorGameEnv { envs, seeds }
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

    #[pyo3(signature = (seed=None, num_airports=None, cash=None))]
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
        let cashes = parse_arg(py, cash, n, vec![650_000.0; n])?;
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
        let c = cash.unwrap_or(650_000.0);
        self.envs[idx] = Game::new(s, num_airports, c);
    }

    #[pyo3(signature = (hours, parallel=None))]
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

    #[pyo3(signature = (hours, mask, parallel=None))]
    fn step_masked(
        &mut self,
        py: Python,
        hours: u64,
        mask: Vec<bool>,
        parallel: Option<bool>,
    ) -> PyResult<()> {
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

    #[pyo3(signature = (cmds, parallel=None))]
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

    /// Vectorized: for each env, returns order IDs at the airport where the plane sits.
    ///
    /// Parameters
    /// ----------
    /// plane_id : usize
    ///     Game plane identifier to query across all environments.
    ///
    /// Returns
    /// -------
    /// Vec[List[int]]
    ///     For each env, a list of order IDs.
    #[pyo3(text_signature = "(plane_id)")]
    fn orders_at_plane_all(&self, plane_id: usize) -> Vec<Vec<usize>> {
        let mut out = Vec::with_capacity(self.envs.len());
        for g in &self.envs {
            if let Some(p) = g.airplanes.iter().find(|p| p.id == plane_id) {
                let loc = p.location;
                if let Some((ap, _)) = g.map.airports.iter().find(|(_, c)| *c == loc) {
                    out.push(ap.orders.iter().map(|o| o.id).collect());
                    continue;
                }
            }
            out.push(Vec::new());
        }
        out
    }

    /// For each env, return the list of airport IDs in that world.
    ///
    /// Returns
    /// -------
    /// Vec[List[int]]
    ///     Airport IDs per environment.
    #[pyo3(text_signature = "()")]
    fn airport_ids_all(&self) -> Vec<Vec<usize>> {
        self.envs
            .iter()
            .map(|g| g.map.airports.iter().map(|(a, _)| a.id).collect())
            .collect()
    }

    #[pyo3(text_signature = "(env_idx, plane_id)")]
    fn sell_plane(&mut self, env_idx: usize, plane_id: usize) -> PyResult<f32> {
        let env = self
            .envs
            .get_mut(env_idx)
            .ok_or_else(|| PyValueError::new_err("env index out of range"))?;
        env.sell_plane(plane_id)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[pymodule]
fn rusty_runways_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GameEnv>()?;
    m.add_class::<VectorGameEnv>()?;
    Ok(())
}
