---
title: Python Bindings
---

# Python

Python bindings expose the Rust Core engine to Python for scripting, analysis, and RL/ML workflows. For game rules, see Core.

As part of the balancing process we routinely let heuristic agents play through procedurally generated worlds so we can observe cash curves, order feasibility, and upgrade cadence in the same way a casual player might experience them. The tooling used for that process lives in the `benchmarks/` directory and is described near the end of this document.

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
g = GameEnv(seed=1, num_airports=5, cash=650_000)
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
- `sell_plane(plane_id: int) -> float`: Sell a parked, empty plane (returns refund).
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
- `sell_plane(env_idx: int, plane_id: int) -> float`: Sell a plane in a specific environment.

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

## Benchmark Agents

The repository contains a small benchmarking harness (see `benchmarks/run_benchmarks.py`) that exercises the Python bindings using a greedy hauling agent. This script evaluates multiple seeds in one go, records cash/fleet/delivery timelines, and renders plots summarizing the progression. To experiment with it locally:

1. Initialise the virtual environment and build the bindings from source:

   ```bash
   ./benchmarks/setup.sh
   source benchmarks/.venv/bin/activate
   ```

2. Run the benchmarking driver:

   ```bash
   python benchmarks/run_benchmarks.py --seeds 0 1 2 --hours 168 --airports 12
   ```

   The script prints per-seed statistics (order feasibility, margin-per-hour, upgrade timing). It also emits CSV timelines and PNG plots under `benchmarks/outputs/` so you can inspect cash/fleet/delivery curves by seed.

For exploratory analysis you can now describe arbitrarily rich batches in YAML and pass them via `--scenario-config`. Each scenario may define:

- the world knobs (`num_airports`, `starting_cash`, `gameplay.orders.*`, etc.); the runner creates temporary YAML configs and cleans them up afterwards;
- sweeps over a single parameter (e.g., vary only `gameplay.restock_cycle_hours`) or named variants with bespoke overrides;
- the seed list and duration for each variant.

All runs end up under `benchmarks/outputs/<scenario>/` alongside a CSV timeline per seed. The driver also produces a `scenario_summary.csv` plus aggregate line/bar charts in `benchmarks/outputs/summary/`, letting you isolate the impact of the swept parameter (restock cadence, order weights, fuel intervals, and so on).

An abridged configuration example:

```yaml
defaults:
  hours: 240
  seeds: [0, 1, 2]
  cash: 650000.0
  num_airports: 12
  gameplay:
    restock_cycle_hours: 168
    fuel_interval_hours: 6
    orders:
      regenerate: true
      generate_initial: true
      max_deadline_hours: 96
      min_weight: 180.0
      max_weight: 650.0
      alpha: 0.12
      beta: 0.55

scenarios:
  - name: baseline
  - name: restock-sweep
    sweep:
      parameter: gameplay.restock_cycle_hours
      values: [96, 120, 168, 240]
  - name: order-weight-variants
    variants:
      - label: light-cargo
        overrides:
          gameplay:
            orders:
              min_weight: 150.0
              max_weight: 600.0
```

See `benchmarks/scenarios.example.yaml` for a more complete walkthrough including external `config_path` worlds.

Feel free to adapt the harness for richer experiments (e.g., alternative agents, different reward functions, or automated regression checks). Because it builds against the local branch, the outputs always reflect the current balance knobs.
