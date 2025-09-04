import numpy as np

from rusty_runways import RustyRunwaysGymEnv, make_sb3_envs
from rusty_runways import RustyRunwaysGymVectorEnv


def test_gym_reset_step():
    env = RustyRunwaysGymEnv(seed=1, num_airports=5, max_hours=5)
    obs, info = env.reset()
    assert isinstance(obs, np.ndarray)
    assert env.observation_space.contains(obs)

    # step a few times with various actions
    for a in [
        np.array([0, 0, 0, 0]),  # ADVANCE
        np.array([1, 0, 0, 0]),  # REFUEL plane 0
        np.array([2, 0, 0, 0]),  # UNLOAD ALL plane 0
        np.array([3, 0, 0, 0]),  # MAINTENANCE plane 0
        np.array([4, 0, 0, 0]),  # DEPART by index
        np.array([5, 0, 0, 0]),  # LOAD ORDER (if any)
    ]:
        obs, reward, terminated, truncated, info = env.step(a)
        assert isinstance(reward, float)
        assert env.observation_space.contains(obs)
        assert not terminated
        assert isinstance(truncated, (bool, np.bool_))


def test_make_thunks():
    thunks = make_sb3_envs(3, seed=1, num_airports=5)
    assert len(thunks) == 3
    # instantiate first env
    e0 = thunks[0]()
    o, info = e0.reset()
    assert isinstance(o, np.ndarray)
    e0.close()


def test_vector_env_basic():
    ven = RustyRunwaysGymVectorEnv(2, seed=1, num_airports=5)
    obs, info = ven.reset()
    assert isinstance(obs, np.ndarray)
    assert obs.shape[0] == 2
    actions = np.array([[0, 0, 0, 0], [1, 0, 0, 0]])
    obs, rewards, terminated, truncated, infos = ven.step(actions)
    assert obs.shape[0] == 2
    assert rewards.shape == (2,)
    assert terminated.shape == (2,)
    assert truncated.shape == (2,)
