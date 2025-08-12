import json
import pytest
from rusty_runways_py import PyGame, PyVectorEnv

def test_create_and_step():
    g = PyGame(seed=1, num_airports=5, cash=1e6)
    g.step(1)
    assert g.time() == 1

def test_state_json():
    g = PyGame(seed=1, num_airports=5, cash=1e6)
    data = json.loads(g.state_json())
    assert all(k in data for k in ["time", "cash", "airports", "planes"])

def test_execute_errors():
    g = PyGame(seed=1, num_airports=5, cash=1e6)
    with pytest.raises(ValueError):
        g.execute("BAD COMMAND")

def test_determinism():
    g1 = PyGame(seed=1, num_airports=5, cash=1e6)
    g1.step(1)
    g2 = PyGame(seed=1, num_airports=5, cash=1e6)
    g2.step(1)
    assert g1.state_json() == g2.state_json()

def test_vector_env_parallel():
    v = PyVectorEnv(8, seed=1, num_airports=5, cash=1e6)
    v.step_all(2, parallel=True)
    assert len(v.states_json()) == 8

def test_full_state_roundtrip():
    g = PyGame(seed=1, num_airports=5, cash=1e6)
    g.step(1)
    dump = g.full_state_json()
    h = PyGame(seed=1, num_airports=5, cash=1e6)
    h.load_full_state_json(dump)
    assert h.time() == g.time()
