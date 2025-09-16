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


def test_reset_with_options_and_all_ops():
    env = RustyRunwaysGymEnv(seed=4, num_airports=6, max_hours=4)
    try:
        env.reset(seed=5, options={"num_airports": 5, "cash": 750000.0})
        # Exercise each operation branch
        env.step(np.array([0, 0, 0, 0]))  # advance
        env.step(np.array([1, 0, 0, 0]))  # refuel
        env.step(np.array([2, 0, 0, 0]))  # unload all
        env.step(np.array([3, 0, 0, 0]))  # maintenance
        env.step(np.array([4, 0, 0, 0]))  # depart
        env.step(np.array([5, 0, 0, 0]))  # load order
        assert env._nearest_other_airport_for_plane0() is not None
        env.step(0)  # scalar branch
        env.step(np.array([1, 0, 0]))  # three-element branch
        env.render()
    finally:
        env.close()


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


def test_vector_env_complex_actions():
    venv = RustyRunwaysGymVectorEnv(2, seed=3, num_airports=6, max_hours=3)
    try:
        venv.reset()
        venv.reset(seed=11)
        acts = np.array([[4, 0, 0, 0], [5, 0, 1, 0]], dtype=int)
        obs, rewards, _, _, _ = venv.step(acts)
        assert obs.shape == (2, 14)
        assert rewards.shape == (2,)
        # cover 1-D action path and helper utilities
        obs2, _, _, _, _ = venv.step(np.array([0, 0, 0, 0, 1, 0, 0, 0]))
        assert obs2.shape == (2, 14)
        proxy = venv._proxy_single(venv._last_states[0])
        assert hasattr(proxy, "_state_cache")
    finally:
        venv.close()


def test_make_sb3_envs_thunks():
    thunks = make_sb3_envs(2, seed=123, num_airports=5)
    assert len(thunks) == 2
    e0 = thunks[0]()
    try:
        obs, info = e0.reset()
        assert isinstance(obs, np.ndarray)
    finally:
        e0.close()
