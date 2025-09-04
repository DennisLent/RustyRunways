import json

from rusty_runways_py import GameEnv, VectorGameEnv


def test_single_env_step():
    g = GameEnv(seed=1)
    g.step(1)
    assert g.time() == 1


def test_execute_success_and_error():
    g = GameEnv(seed=1)
    g.execute("ADVANCE 1")
    assert g.time() == 1
    try:
        g.execute("BADCMD")
    except ValueError:
        pass
    else:
        assert False


def test_state_json_schema():
    g = GameEnv(seed=1, num_airports=2)
    data = json.loads(g.state_json())
    assert {"time", "cash", "airports", "planes"}.issubset(data.keys())


def test_full_state_roundtrip():
    g = GameEnv(seed=1)
    g.step(2)
    dump = g.full_state_json()
    g2 = GameEnv(seed=2)
    g2.load_full_state_json(dump)
    assert g2.time() == g.time()
    assert g2.cash() == g.cash()


def test_vector_env_basic():
    env = VectorGameEnv(4, seed=1)
    env.step_all(2, parallel=True)
    assert env.times() == [2, 2, 2, 2]


def test_vector_env_execute_all():
    env = VectorGameEnv(2, seed=1)
    res = env.execute_all(["ADVANCE 1", "BAD"], parallel=False)
    assert res[0][0] is True
    assert res[1][0] is False and res[1][1] is not None


def test_vector_env_reset_all():
    env = VectorGameEnv(2, seed=1)
    env.step_all(1, parallel=False)
    env.reset_all(seed=2)
    assert env.times() == [0, 0]
    assert env.seeds() == [2, 3]


def test_vector_env_masked_steps():
    env = VectorGameEnv(4, seed=1)
    env.step_masked(1, [True, False, True, False])
    assert env.times() == [1, 0, 1, 0]


def test_determinism():
    g1 = GameEnv(seed=42)
    g2 = GameEnv(seed=42)
    g1.execute("ADVANCE 5")
    g2.execute("ADVANCE 5")
    assert g1.state_json() == g2.state_json()

    env1 = VectorGameEnv(2, seed=3)
    env2 = VectorGameEnv(2, seed=3)
    env1.step_all(1, parallel=True)
    env2.step_all(1, parallel=True)
    assert env1.state_all_json() == env2.state_all_json()


def test_reset_resets_state():
    g = GameEnv(seed=1, num_airports=3, cash=500.0)
    g.step(5)
    g.reset(seed=2, num_airports=4, cash=600.0)
    assert g.time() == 0
    assert g.cash() == 600.0
    assert g.seed() == 2


def test_vector_env_parallel_vs_serial():
    parallel = VectorGameEnv(2, seed=1)
    serial = VectorGameEnv(2, seed=1)
    parallel.step_all(3, parallel=True)
    serial.step_all(3, parallel=False)
    assert parallel.times() == serial.times() == [3, 3]
    assert parallel.cashes() == serial.cashes()


def test_vector_env_reset_all_mismatch():
    env = VectorGameEnv(2, seed=1)
    import pytest

    with pytest.raises(ValueError):
        env.reset_all(seed=[1, 2, 3])


def test_vector_env_step_masked_length_error():
    env = VectorGameEnv(2, seed=1)
    import pytest

    with pytest.raises(ValueError):
        env.step_masked(1, [True])


def test_vector_env_execute_all_none():
    env = VectorGameEnv(2, seed=1)
    res = env.execute_all([None, "ADVANCE 1"], parallel=False)
    assert res[0] == (True, None)
    assert res[1][0] is True and res[1][1] is None
    assert env.times() == [0, 1]


def test_vector_env_reset_at_only_one():
    env = VectorGameEnv(2, seed=1)
    env.step_all(2, parallel=False)
    env.reset_at(0, seed=5, num_airports=3, cash=700.0)
    assert env.times() == [0, 2]
    assert env.seeds() == [5, 2]


def test_step_zero_noop():
    g = GameEnv(seed=1)
    g.step(0)
    assert g.time() == 0


def test_vector_env_execute_all_parallel():
    env = VectorGameEnv(2, seed=1)
    res = env.execute_all(["ADVANCE 1", "ADVANCE 1"], parallel=True)
    assert res == [(True, None), (True, None)]
    assert env.times() == [1, 1]


def test_vector_env_reset_at_invalid_index():
    env = VectorGameEnv(2, seed=1)
    import pytest

    with pytest.raises(Exception):
        env.reset_at(5, seed=1)


def test_vector_env_step_all_zero():
    env = VectorGameEnv(3, seed=1)
    env.step_all(0, parallel=False)
    assert env.times() == [0, 0, 0]
