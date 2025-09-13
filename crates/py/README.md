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

## Links

- Documentation: https://dennislent.github.io/RustyRunways
- Source: https://github.com/DennisLent/RustyRunways
- Issues: https://github.com/DennisLent/RustyRunways/issues

License: MIT
