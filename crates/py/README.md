<!-- PyPI long description lives here. Keep links absolute. -->

# Rusty Runways — Python Bindings

[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://dennislent.github.io/RustyRunways)
[![PyPI](https://img.shields.io/pypi/v/rusty-runways.svg)](https://pypi.org/project/rusty-runways/)

<p align="center">
  <img src="https://github.com/DennisLent/RustyRunways/raw/main/docs/assets/rusty_runways.png" alt="Rusty Runways" width="640" />
</p>

Deterministic airline logistics simulation written in Rust with a rich Python API for scripting, analysis, and RL/ML. Includes fast vectorized environments and optional Gymnasium wrappers.

— Full docs: https://dennislent.github.io/RustyRunways

## Install

Python (PyPI):

```bash
pip install rusty-runways

# Optional Gym wrappers
pip install 'rusty-runways[gym]'
```

Local dev (build from source):

```bash
cd crates/py
maturin develop --release
```

## Quick Start (Python)

Engine bindings (single and vector):

```python
from rusty_runways_py import GameEnv, VectorGameEnv

g = GameEnv(seed=1, num_airports=5, cash=1_000_000)
g.step(1)
print(g.time(), g.cash())

venv = VectorGameEnv(4, seed=1)
venv.step_all(1, parallel=True)
print(venv.times())
```

Notes:
- Seeds control determinism.
- `VectorGameEnv.step_all(..., parallel=True)` releases the GIL and uses Rayon under the hood.

## Gymnasium Wrappers (optional)

Wrappers live in the pure‑Python package `rusty_runways` and require `gymnasium`:

```python
from stable_baselines3 import PPO
from stable_baselines3.common.vec_env import DummyVecEnv
from rusty_runways import RustyRunwaysGymEnv, make_sb3_envs

# Single‑env Gym
env = RustyRunwaysGymEnv(seed=1, num_airports=5)

# SB3 convenience (DummyVecEnv)
vec_env = DummyVecEnv(make_sb3_envs(4, seed=1, num_airports=5))
model = PPO("MlpPolicy", vec_env, verbose=1)
model.learn(total_timesteps=10_000)
```

## YAML Worlds and Custom Airplanes

All constructors accept a `config_path` pointing to a YAML world. The YAML can optionally define an `airplanes` catalog to replace or extend the built‑in models. Example:

```yaml
seed: 42
starting_cash: 650000.0
airports: [ ... ]
airplanes:
  strategy: replace   # or: add (default)
  models:
    - name: WorkshopCombi
      mtow: 15000.0
      cruise_speed: 520.0
      fuel_capacity: 3200.0
      fuel_consumption: 260.0
      operating_cost: 950.0
      payload_capacity: 3200.0
      passenger_capacity: 24
      purchase_price: 780000.0
      min_runway_length: 1200.0
      role: Mixed        # Cargo | Passenger | Mixed
```

From Python:

```python
from rusty_runways_py import GameEnv

g = GameEnv(config_path="examples/sample_world.yaml")
# List the models the current game exposes (custom + built‑ins depending on strategy)
print(g.models_py())       # Python list of dicts
print(g.models_json())     # JSON string
```

Validation ensures all fields are present and role constraints hold (Cargo requires payload_capacity > 0; Passenger requires passenger_capacity > 0; Mixed requires both).

## Links

- Documentation: https://dennislent.github.io/RustyRunways
- Source: https://github.com/DennisLent/RustyRunways
- Issues: https://github.com/DennisLent/RustyRunways/issues

License: MIT
