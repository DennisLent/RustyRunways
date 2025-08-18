import json

from rusty_runways_py import PyGame, PyVectorEnv


def test_single_env_step():
    g = PyGame(seed=1)
    g.step(1)
    assert g.time() == 1


def test_execute_success_and_error():
    g = PyGame(seed=1)
    g.execute("ADVANCE 1")
    assert g.time() == 1
    try:
        g.execute("BADCMD")
    except ValueError:
        pass
    else:
        assert False


def test_state_json_schema():
    g = PyGame(seed=1, num_airports=2)
    data = json.loads(g.state_json())
    assert {"time", "cash", "airports", "planes"}.issubset(data.keys())


def test_full_state_roundtrip():
    g = PyGame(seed=1)
    g.step(2)
    dump = g.full_state_json()
    g2 = PyGame(seed=2)
    g2.load_full_state_json(dump)
    assert g2.time() == g.time()
    assert g2.cash() == g.cash()


def test_vector_env_basic():
    env = PyVectorEnv(4, seed=1)
    env.step_all(2, parallel=True)
    assert env.times() == [2, 2, 2, 2]


def test_vector_env_execute_all():
    env = PyVectorEnv(2, seed=1)
    res = env.execute_all(["ADVANCE 1", "BAD"], parallel=False)
    assert res[0][0] is True
    assert res[1][0] is False and res[1][1] is not None


def test_vector_env_reset_all():
    env = PyVectorEnv(2, seed=1)
    env.step_all(1, parallel=False)
    env.reset_all(seed=2)
    assert env.times() == [0, 0]
    assert env.seeds() == [2, 3]


def test_vector_env_masked_steps():
    env = PyVectorEnv(4, seed=1)
    env.step_masked(1, [True, False, True, False])
    assert env.times() == [1, 0, 1, 0]


def test_determinism():
    g1 = PyGame(seed=42)
    g2 = PyGame(seed=42)
    g1.execute("ADVANCE 5")
    g2.execute("ADVANCE 5")
    assert g1.state_json() == g2.state_json()

    env1 = PyVectorEnv(2, seed=3)
    env2 = PyVectorEnv(2, seed=3)
    env1.step_all(1, parallel=True)
    env2.step_all(1, parallel=True)
    assert env1.state_all_json() == env2.state_all_json()

