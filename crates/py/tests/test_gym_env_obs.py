import numpy as np

from rusty_runways import gym_env


def test_safe_mean_empty():
    assert gym_env._safe_mean([]) == 0.0


def test_build_obs_from_state_basic():
    s = {
        "time": 5,
        "cash": 123.0,
        "airports": [
            {
                "id": 0,
                "x": 0.0,
                "y": 0.0,
                "fuel_price": 2.0,
                "runway_length": 1000.0,
                "num_orders": 2,
            },
            {
                "id": 1,
                "x": 1.0,
                "y": 1.0,
                "fuel_price": 4.0,
                "runway_length": 1500.0,
                "num_orders": 3,
            },
        ],
        "planes": [
            {
                "id": 0,
                "x": 0.0,
                "y": 0.0,
                "status": "Parked",
                "fuel": {"current": 10.0, "capacity": 20.0},
                "payload": {"current": 1.0, "capacity": 4.0},
                "hours_remaining": 0,
            }
        ],
    }

    vec = gym_env._build_obs_from_state(s)
    assert isinstance(vec, np.ndarray)
    assert vec.shape == (14,)

    # Global features
    assert vec[0] == 5.0  # time
    assert vec[1] == 123.0  # cash
    assert vec[2] == 2.0  # n_airports
    assert vec[3] == 1.0  # n_planes
    assert vec[4] == 5.0  # total_orders
    assert np.isclose(vec[5], 3.0)  # avg_fuel
    assert np.isclose(vec[6], 1250.0)  # avg_runway

    # Plane 0 features
    assert np.isclose(vec[7], 0.5)  # fuel ratio
    assert np.isclose(vec[8], 0.25)  # payload ratio
    assert vec[9] == 0.0  # status code parked
    assert vec[10] == 0.0  # hours remaining

    # Current airport features (match plane 0 coords to airport 0)
    assert vec[11] == 2.0  # fuel price at current airport
    assert vec[12] == 1000.0  # runway length at current airport
    assert vec[13] == 2.0  # orders at current airport
