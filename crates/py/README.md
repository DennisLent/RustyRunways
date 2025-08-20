# Rusty Runways Python Bindings

## Install

```bash
maturin develop --release
```

## Quick Usage

```python
from rusty_runways_py import PyGame, PyVectorEnv

g = PyGame(seed=1)
g.step(1)
print(g.time(), g.cash())

env = PyVectorEnv(4, seed=1)
env.step_all(1, parallel=True)
print(env.times())
```

Deterministic behaviour is controlled by seeds. Parallel stepping releases the GIL and uses rayon under the hood.
