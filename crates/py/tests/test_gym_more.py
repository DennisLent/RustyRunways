import numpy as np

from rusty_runways import RustyRunwaysGymEnv, RustyRunwaysGymVectorEnv, make_sb3_envs


def test_single_env_spaces_and_step_shapes():
    env = RustyRunwaysGymEnv(seed=1, num_airports=5)
    obs, info = env.reset()
    assert isinstance(obs, np.ndarray)
    assert obs.shape == (14,)
    # action space is MultiDiscrete([6, 16, 64, 256])
    nvec = list(env.action_space.nvec)  # type: ignore[attr-defined]
    assert nvec[:3] == [6, 16, 64]
    assert nvec[3] >= 1  # allow different max airports in future

    # basic step
    action = np.array([0, 0, 0, 0])
    obs, reward, terminated, truncated, info = env.step(action)
    assert isinstance(obs, np.ndarray)
    assert isinstance(reward, float)
    assert isinstance(terminated, (bool, np.bool_))
    assert isinstance(truncated, (bool, np.bool_))


def test_custom_reward_fn_is_used():
    def rwd(curr, prev):
        return -1.0

    env = RustyRunwaysGymEnv(seed=1, num_airports=5, reward_fn=rwd)
    env.reset()
    _, reward, *_ = env.step(np.array([0, 0, 0, 0]))
    assert reward == -1.0


def test_vector_env_truncation_and_shapes():
    venv = RustyRunwaysGymVectorEnv(3, seed=1, num_airports=5, max_hours=2)
    obs, info = venv.reset()
    assert obs.shape == (3, 14)
    acts = np.zeros((3, 4), dtype=int)
    _, rewards, term, trunc, infos = venv.step(acts)
    assert rewards.shape == (3,)
    assert term.shape == (3,)
    assert trunc.shape == (3,)
    # after second step, truncates due to max_hours=2
    _, _, _, trunc2, _ = venv.step(acts)
    assert trunc2.dtype == bool
    assert trunc2.tolist() == [True, True, True]


def test_make_sb3_envs_thunks():
    thunks = make_sb3_envs(2, seed=123, num_airports=5)
    assert len(thunks) == 2
    e0 = thunks[0]()
    try:
        obs, info = e0.reset()
        assert isinstance(obs, np.ndarray)
    finally:
        e0.close()
