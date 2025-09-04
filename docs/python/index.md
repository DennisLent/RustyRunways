---
title: Python Bindings
---

# Python

Python bindings (`rusty_runways_py`) expose the Core engine to Python for scripting, prototyping, and RL/ML workflows. For game rules, see [Core](../core/index.md).

## Installation

Build and install locally with `maturin`:

```bash
cd crates/py
maturin develop --release
```

## Environments

Two classes are exported:

- `GameEnv` — single environment wrapping a `Game`.
- `VectorGameEnv` — a batch of `Game` instances, with optional parallel stepping using Rayon and GIL‑friendly APIs.

### GameEnv

Constructor signature:

```python
GameEnv(seed: int | None = None, num_airports: int | None = None, cash: float | None = None)
```

Example:

```python
from rusty_runways_py import GameEnv

g = GameEnv(seed=1, num_airports=5, cash=1_000_000)
g.step(1)
print(g.time(), g.cash())
print(g.drain_log())
```

Notable methods (selection):

- `step(hours: int)` — advance time.
- `execute(cmd: str)` — run a CLI command (see CLI docs for syntax).
- `state_json()` / `full_state_json()` — JSON snapshots.
- `time()`, `cash()`, `log()`, `drain_log()` — observability helpers.

### VectorGameEnv

Constructor signature:

```python
VectorGameEnv(
    n_envs: int,
    seed: int | None = None,
    num_airports: int | None = None,
    cash: float | None = None,
)
```

Key methods (selection):

- `env_count()` / `__len__()` — number of envs.
- `seeds()` — per‑env seeds (base seed + index if a single seed was provided).
- `reset_all(seed=None, num_airports=None, cash=None)` — vectorized reset; each arg can be scalar or per‑env list.
- `step_all(hours, parallel=True)` — step all environments; uses Rayon if `parallel=True`.
- `execute_all(cmds, parallel=True)` — run CLI commands across envs; `cmds` is a list of strings per env.
- `times()`, `cashes()`, `logs()`, `drain_logs()` — vectorized observability.

Example (parallel stepping):

```python
from rusty_runways_py import VectorGameEnv

env = VectorGameEnv(4, seed=1)
env.step_all(1, parallel=True)
print(env.times())
print(env.cashes())
```

Example (serial stepping for determinism checks):

```python
parallel = VectorGameEnv(2, seed=1)
serial = VectorGameEnv(2, seed=1)
parallel.step_all(3, parallel=True)
serial.step_all(3, parallel=False)
assert parallel.times() == serial.times() == [3, 3]
assert parallel.cashes() == serial.cashes()
```

## Notes

- All rules and constraints match the Rust Core engine.
- Seeds control determinism. Vector environments default to `base_seed + index` if a single seed is provided.
- Parallel operations release the GIL and use Rayon internally.

