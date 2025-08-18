import json
from rusty_runways_py import PyGame

def test_create_and_step():
    g = PyGame(seed=1, num_airports=5, cash=1000.0)
    g.step(1)
    assert g.time() == 1
    g.step(1)
    assert g.time() == 2

def test_observation_schema():
    g = PyGame(seed=1, num_airports=2, cash=1000.0)
    data = json.loads(g.state_json())
    assert "time" in data and "cash" in data
    assert isinstance(data["airports"], list)
    assert isinstance(data["planes"], list)

def test_execute_round_trip():
    g = PyGame(seed=1, num_airports=2, cash=1000.0)
    g.execute("ADVANCE 3")
    assert g.time() == 3

def test_determinism():
    g1 = PyGame(seed=42)
    g2 = PyGame(seed=42)
    g1.execute("ADVANCE 5")
    g2.execute("ADVANCE 5")
    assert g1.state_json() == g2.state_json()
