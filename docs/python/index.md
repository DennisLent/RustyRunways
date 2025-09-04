---
title: Python Bindings
---

# Python

Python bindings expose the Rust Core engine to Python for scripting, analysis, and RL/ML workflows. For game rules, see Core.

## Installation

- PyPI (recommended):

```bash
pip install rusty-runways
```

- With Gymnasium support (for Gym wrappers):

```bash
pip install 'rusty-runways[gym]'
```

- Local dev build:

```bash
cd crates/py
maturin develop --release
```

## Imports at a Glance

- Engine bindings: `from rusty_runways_py import GameEnv, VectorGameEnv`
- Gym wrappers: `from rusty_runways import RustyRunwaysGymEnv, RustyRunwaysGymVectorEnv, make_sb3_envs`

Gymnasium is only required for the Gym wrappers. See the Gym section for details.

## GameEnv (single)

Constructor

```python
GameEnv(seed: int | None = None,
        num_airports: int | None = None,
        cash: float | None = None,
        config_path: str | None = None)
```

Common usage

```python
from rusty_runways_py import GameEnv

# Start from a seeded random world
g = GameEnv(seed=1, num_airports=5, cash=1_000_000)
g.step(1)
print(g.time(), g.cash())
print(g.drain_log())

# Start from a custom world YAML
g2 = GameEnv(config_path="examples/sample_world.yaml")
print(g2.time(), g2.cash())
```

Key methods

- `reset(seed=None, num_airports=None, cash=None, config_path=None)`: Reinitialize the world.
- `step(hours: int)`: Advance simulation time by `hours`.
- `execute(cmd: str)`: Run CLI command (see CLI docs for syntax).
- `state_json() -> str`: JSON snapshot of the observable state.
- `state_py() -> dict`: Python dict snapshot (JSON decoded).
- `full_state_json() -> str`: Full internal state snapshot.
- `load_full_state_json(s: str)`: Restore full internal state snapshot.
- `time() -> int`, `cash() -> float`, `seed() -> int`.
- `drain_log() -> list[str]`: Retrieve and clear sim log.
- `orders_at_plane(plane_id: int) -> list[int]`: Order IDs available at that plane’s airport.
- `airport_ids() -> list[int]`: All airport IDs in the world.

Inspecting state

```python
obs = g.state_py()
print(obs["airports"][0])
print(obs["planes"][0])
```

## VectorGameEnv (multiple)

Constructor

```python
VectorGameEnv(n_envs: int,
              seed: int | None = None,
              num_airports: int | None = None,
              cash: float | None = None,
              config_path: str | None = None)
```

Core vector API

- `env_count() / __len__()`: Number of envs.
- `seeds() -> list[int]`: Per‑env seeds.
- `reset_all(seed=None, num_airports=None, cash=None)`: Vector reset; values can be scalars or lists.
- `reset_at(idx, seed=None, num_airports=None, cash=None)`: Reset a single env.
- `step_all(hours, parallel=True)`: Advance all envs (Rayon‑parallel when `parallel=True`).
- `step_masked(hours, mask, parallel=True)`: Advance a subset by boolean mask.
- `execute_all(cmds, parallel=True) -> list[tuple[bool, Optional[str]]]`: Run a command (or `None`) per env.
- `state_all_json() / state_all_py()`: Vector snapshots.
- `times() -> list[int]`, `cashes() -> list[float]`, `drain_logs() -> list[list[str]]`.
- `orders_at_plane_all(plane_id) -> list[list[int]]`, `airport_ids_all() -> list[list[int]]`.

Examples

```python
from rusty_runways_py import VectorGameEnv

env = VectorGameEnv(4, seed=1)
env.step_all(1, parallel=True)
print(env.times())        # [1, 1, 1, 1]
print(env.cashes())

# Determinism check: parallel vs serial stepping
serial = VectorGameEnv(4, seed=1)
env.step_all(3, parallel=True)
serial.step_all(3, parallel=False)
assert env.times() == serial.times() == [4, 4, 4, 4]

# Start from a custom world for all envs
env2 = VectorGameEnv(4, config_path="examples/sample_world.yaml")
print(env2.times())
```

## Gymnasium Wrappers

Wrappers live under the pure‑Python package `rusty_runways` and require `gymnasium`:

- `RustyRunwaysGymEnv`: Single‑env wrapper over `GameEnv`.
- `RustyRunwaysGymVectorEnv`: Vector wrapper over `VectorGameEnv` (implements `gym.vector.VectorEnv`).
- `make_sb3_envs(n_envs, seed=None, **kwargs)`: Convenience to build `DummyVecEnv`/`SubprocVecEnv` inputs for SB3.

Observation and action spaces

- Observation: `Box(float32, shape=(14,))` summary features derived from state JSON.
- Action: `MultiDiscrete([6, 16, 64, 256])` encoding `[op, plane_id, selector, dest_index]` where `op` in
  0 ADVANCE, 1 REFUEL, 2 UNLOAD_ALL, 3 MAINTENANCE, 4 DEPART_TO_INDEX, 5 LOAD_ORDER.
- Reward: By default, delta cash per step; can be customized with `reward_fn(state, prev_state)` on the single‑env wrapper.

Single‑env example

```python
from rusty_runways import RustyRunwaysGymEnv

env = RustyRunwaysGymEnv(seed=1, num_airports=5)
obs, info = env.reset()
for _ in range(10):
    action = env.action_space.sample()
    obs, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        obs, info = env.reset()
```

Vector (Gym VectorEnv) example

```python
from rusty_runways import RustyRunwaysGymVectorEnv

venv = RustyRunwaysGymVectorEnv(8, seed=123, num_airports=5)
obs, info = venv.reset()
acts = venv.action_space.sample()
venv.step_async(acts)
obs, rewards, terminated, truncated, infos = venv.step_wait()
```

Stable‑Baselines3 example

```python
from stable_baselines3 import PPO
from stable_baselines3.common.vec_env import DummyVecEnv
from rusty_runways import make_sb3_envs

vec_env = DummyVecEnv(make_sb3_envs(4, seed=1, num_airports=5))
model = PPO("MlpPolicy", vec_env, verbose=1)
model.learn(total_timesteps=10_000)
```

Dependency note

- Install via extra for full features: `pip install 'rusty-runways[gym]'`.
- Or install directly: `pip install gymnasium`.
- If Gymnasium is not installed, attempting to use the wrappers will raise a helpful ImportError explaining how to enable them.

## Notes

- Same rules/constraints as the Rust Core engine.
- Seeds control determinism; vectors default to `base_seed + index` when provided a scalar seed.
- Parallel stepping releases the GIL and uses Rayon internally.
