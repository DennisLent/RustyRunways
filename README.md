# RustyRunways

[![codecov](https://codecov.io/github/DennisLent/RustyRunways/graph/badge.svg?token=NVMX1JW002)](https://codecov.io/github/DennisLent/RustyRunways)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://dennislent.github.io/RustyRunways)
[![PyPI](https://img.shields.io/pypi/v/rusty-runways.svg)](https://pypi.org/project/rusty-runways/)

![Rusty Runways](docs/assets/rusty_runways.png)

RustyRunways is a deterministic airline logistics simulation built in Rust with a CLI, GUI, and Python APIs. It’s designed for AI/ML/RL: train agents, prototype policies, and run fast, reproducible experiments in a realistic, constraint‑driven game world.

Keywords: AI, machine learning, reinforcement learning, RL environment, Gymnasium, Stable‑Baselines3, vectorized environments, simulation game, Rust game engine, Python bindings, deterministic seeds, parallel stepping.

---

## Install

- Python (PyPI):

```bash
pip install rusty-runways
# with Gym wrappers
pip install 'rusty-runways[gym]'
```

- Build from source (Rust):

```bash
cargo build --release
```

Docs for CLI, GUI, and Python: https://dennislent.github.io/RustyRunways

---

## Quick Start (Python)

Engine bindings (single + vector envs):

```python
from rusty_runways_py import GameEnv, VectorGameEnv

g = GameEnv(seed=1, num_airports=5)
g.step(1)
print(g.time(), g.cash())

venv = VectorGameEnv(4, seed=1)
venv.step_all(1, parallel=True)
print(venv.times())
```

Gym wrappers (optional, install extra):

```python
from stable_baselines3 import PPO
from stable_baselines3.common.vec_env import DummyVecEnv
from rusty_runways import RustyRunwaysGymEnv, make_sb3_envs

vec_env = DummyVecEnv(make_sb3_envs(4, seed=1, num_airports=5))
model = PPO("MlpPolicy", vec_env, verbose=1)
model.learn(total_timesteps=10_000)
```

See Python docs for observation/action spaces, reward shaping, and vectorized stepping.

---

## Why RustyRunways for RL/AI

- Deterministic seeds: reproducible training and evaluation.
- Vectorized environments: fast rollouts with optional parallel stepping.
- Compact action surface (refuel/load/unload/depart/advance/maintenance) and rich JSON observations.
- Real‑world constraints (runway length, fuel costs, payload, deadlines) ideal for decision‑making under constraints.
- Multi‑frontend: CLI for scripting, GUI for inspection, Python for training and research.

---

## Links

- Docs: https://dennislent.github.io/RustyRunways
- PyPI: https://pypi.org/project/rusty-runways/
- Issues: https://github.com/DennisLent/RustyRunways/issues

Contributions welcome! Open issues and PRs for bug fixes, features, or RL tooling.

