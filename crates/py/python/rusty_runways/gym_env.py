from __future__ import annotations

import json
from typing import Any, Callable, List, Optional, Tuple

import gymnasium as gym
import numpy as np
from gymnasium import spaces

from rusty_runways_py import GameEnv, VectorGameEnv


def _safe_mean(xs: List[float]) -> float:
    """Mean with empty-safe fallback.

    Parameters
    ----------
    xs : list[float]
        Sequence of numeric values.

    Returns
    -------
    float
        Mean of values or 0.0 if empty.
    """
    return float(np.mean(xs)) if xs else 0.0


def _build_obs_from_state(s: dict) -> np.ndarray:
    """Build a fixed-size observation vector from a Game observation dict.

    Parameters
    ----------
    s : dict
        JSON-decoded dictionary returned by ``GameEnv.state_json()``.

    Returns
    -------
    np.ndarray
        Feature vector of shape ``(14,)`` and dtype ``float32``.
    """
    time_h = float(s.get("time", 0))
    cash = float(s.get("cash", 0.0))
    airports = s.get("airports", [])
    planes = s.get("planes", [])

    n_airports = float(len(airports))
    n_planes = float(len(planes))
    total_orders = float(sum(a.get("num_orders", 0) for a in airports))
    avg_fuel = _safe_mean([float(a.get("fuel_price", 0.0)) for a in airports])
    avg_runway = _safe_mean([float(a.get("runway_length", 0.0)) for a in airports])

    p0 = None
    for p in planes:
        if int(p.get("id", -1)) == 0:
            p0 = p
            break
    p0_fuel_ratio = p0_payload_ratio = 0.0
    p0_status_code = 0.0
    hours_remaining = 0.0
    curr_ap_fuel = curr_ap_runway = curr_ap_orders = 0.0

    if p0 is not None:
        fuel = p0.get("fuel", {})
        payload = p0.get("payload", {})
        cf = float(fuel.get("current", 0.0))
        capf = float(fuel.get("capacity", 1.0))
        p0_fuel_ratio = cf / capf if capf > 0 else 0.0
        cp = float(payload.get("current", 0.0))
        capp = float(payload.get("capacity", 1.0))
        p0_payload_ratio = cp / capp if capp > 0 else 0.0

        status = str(p0.get("status", "Parked")).lower()
        status_map = {
            "parked": 0.0,
            "refueling": 1.0,
            "loading": 2.0,
            "unloading": 3.0,
            "maintenance": 4.0,
            "intransit": 5.0,
            "broken": 6.0,
        }
        status_key = status.replace(" ", "").replace("_", "")
        p0_status_code = status_map.get(status_key, 0.0)
        hr = p0.get("hours_remaining")
        hours_remaining = float(hr) if hr is not None else 0.0

        px, py = float(p0.get("x", 0.0)), float(p0.get("y", 0.0))
        for a in airports:
            if abs(float(a.get("x", 0.0)) - px) < 1e-6 and abs(float(a.get("y", 0.0)) - py) < 1e-6:
                curr_ap_fuel = float(a.get("fuel_price", 0.0))
                curr_ap_runway = float(a.get("runway_length", 0.0))
                curr_ap_orders = float(a.get("num_orders", 0.0))
                break

    vec = np.array(
        [
            time_h,
            cash,
            n_airports,
            n_planes,
            total_orders,
            avg_fuel,
            avg_runway,
            p0_fuel_ratio,
            p0_payload_ratio,
            p0_status_code,
            hours_remaining,
            curr_ap_fuel,
            curr_ap_runway,
            curr_ap_orders,
        ],
        dtype=np.float32,
    )
    return vec


class RustyRunwaysGymEnv(gym.Env):
    """Gymnasium wrapper for a single RustyRunways `GameEnv`.

    Parameters
    ----------
    seed : int | None
        RNG seed for deterministic world generation.
    num_airports : int | None
        Number of airports (if not using config_path).
    cash : float | None
        Starting cash.
    config_path : str | None
        YAML config path for a custom world. If provided, it overrides seed/num_airports.
    max_hours : int, default=1000
        Episode truncation horizon in hours.
    reward_fn : Callable[[dict, dict], float] | None
        Optional custom reward: takes (state, prev_state) dicts from state_json.

    Observation
    -----------
    Box(float32, shape=(14,)). Features include global stats and plane 0 summary.

    Action
    ------
    MultiDiscrete([6, 16, 64, 256]) encoding (op, plane_id, selector, dest_index):
    0 ADVANCE, 1 REFUEL, 2 UNLOAD_ALL, 3 MAINTENANCE, 4 DEPART_BY_INDEX, 5 LOAD_ORDER.
    """

    metadata = {"render.modes": []}

    def __init__(
        self,
        seed: Optional[int] = None,
        num_airports: Optional[int] = None,
        cash: Optional[float] = None,
        config_path: Optional[str] = None,
        max_hours: int = 1000,
        reward_fn: Optional[Callable[[dict, dict], float]] = None,
    ) -> None:
        super().__init__()
        self._params = dict(
            seed=seed,
            num_airports=num_airports,
            cash=cash,
            config_path=config_path,
        )
        self._env = GameEnv(**self._params)  # type: ignore[arg-type]
        self._elapsed = 0
        self.max_hours = int(max_hours)
        self._reward_fn = reward_fn

        obs = self._observe()
        self.observation_space = spaces.Box(
            low=-np.inf,
            high=np.inf,
            shape=obs.shape,
            dtype=np.float32,
        )
        # Actions encoded as MultiDiscrete([N_OPS, MAX_PLANES, MAX_SELECT, MAX_AIRPORTS])
        # N_OPS:
        #   0 ADVANCE 1h
        #   1 REFUEL plane
        #   2 UNLOAD ALL plane
        #   3 MAINTENANCE plane
        #   4 DEPART plane to airport selected by index
        #   5 LOAD ORDER (selector indexes order list at plane's airport)
        self.N_OPS = 6
        self.MAX_PLANES = 16
        self.MAX_SELECT = 64
        self.MAX_AIRPORTS = 256
        self.action_space = spaces.MultiDiscrete(
            [self.N_OPS, self.MAX_PLANES, self.MAX_SELECT, self.MAX_AIRPORTS]
        )

        self._last_cash = float(self._state_cache.get("cash", 0.0))

    # ----------- Gym API -----------
    def reset(
        self,
        *,
        seed: Optional[int] = None,
        options: Optional[dict] = None,
    ) -> Tuple[np.ndarray, dict]:
        """Reset the environment.

        Parameters
        ----------
        seed : int | None
            Optional seed override.
        options : dict | None
            May contain keys among {"seed", "num_airports", "cash", "config_path"}.

        Returns
        -------
        obs : np.ndarray
            Initial observation.
        info : dict
            Empty dict for API compatibility.
        """
        if options is None:
            options = {}
        params = self._params.copy()
        if seed is not None:
            params["seed"] = seed
        # allow passing overrides via options
        params.update(
            {
                k: v
                for k, v in options.items()
                if k in ("seed", "num_airports", "cash", "config_path")
            }
        )
        self._env.reset(**params)  # type: ignore[arg-type]
        self._elapsed = 0
        obs = self._observe()
        self._last_cash = float(self._state_cache.get("cash", 0.0))
        return obs, {}

    def step(self, action) -> Tuple[np.ndarray, float, bool, bool, dict]:
        """Step the environment by one hour.

        Parameters
        ----------
        action : array-like | int
            Encoded as MultiDiscrete([op, plane, selector]).

        Returns
        -------
        obs : np.ndarray
            Next observation.
        reward : float
            Delta cash (or custom reward) for this transition.
        terminated : bool
            Always False (no terminal condition yet).
        truncated : bool
            True when episode reaches `max_hours`.
        info : dict
            Empty dict.
        """
        arr = np.asarray(action).astype(int)
        if arr.ndim == 0:
            op, plane, sel, dest_idx = int(arr), 0, 0, 0
        elif arr.size >= 4:
            op, plane, sel, dest_idx = int(arr[0]), int(arr[1]), int(arr[2]), int(arr[3])
        elif arr.size == 3:
            op, plane, sel, dest_idx = int(arr[0]), int(arr[1]), int(arr[2]), 0
        else:
            op, plane, sel, dest_idx = int(arr.flat[0]), 0, 0, 0

        if op == 0:
            self._env.step(1)
            self._elapsed += 1
        elif op == 1:
            # Refuel plane 0
            try:
                self._env.execute(f"REFUEL PLANE {plane}")
            except Exception:
                pass
            self._env.step(1)
            self._elapsed += 1
        elif op == 2:
            try:
                self._env.execute(f"UNLOAD ALL FROM {plane}")
            except Exception:
                pass
            self._env.step(1)
            self._elapsed += 1
        elif op == 3:
            try:
                self._env.execute(f"MAINTENANCE {plane}")
            except Exception:
                pass
            self._env.step(1)
            self._elapsed += 1
        elif op == 4:
            # depart to selected airport id if available
            try:
                ids = self._env.airport_ids()
                # exclude current airport id
                curr_id = self._current_airport_id_for_plane(plane)
                ids = [i for i in ids if i != curr_id]
                dest = ids[dest_idx % len(ids)] if ids else None
            except Exception:
                dest = None
            if dest is not None:
                try:
                    self._env.execute(f"DEPART PLANE {plane} {dest}")
                except Exception:
                    pass
                # let time tick for departure handling
                self._env.step(1)
                self._elapsed += 1
        elif op == 5:
            # LOAD ORDER based on selector index
            try:
                ids = self._env.orders_at_plane(plane)
                if ids:
                    order_id = ids[sel % len(ids)]
                    self._env.execute(f"LOAD ORDER {order_id} ON {plane}")
            except Exception:
                pass
            self._env.step(1)
            self._elapsed += 1

        obs = self._observe()
        cash = float(self._state_cache.get("cash", 0.0))
        if self._reward_fn is not None:
            reward = float(self._reward_fn(self._state_cache, self._prev_state_cache))
        else:
            reward = float(cash - self._last_cash)
        self._last_cash = cash

        terminated = False
        truncated = self._elapsed >= self.max_hours
        info = {}
        return obs, reward, terminated, truncated, info

    def render(self) -> None:
        """No-op renderer (use the GUI binary for visualization)."""
        return None

    # ----------- Helpers -----------
    def _observe(self) -> np.ndarray:
        """Compute observation vector from current state_json."""
        state_json = self._env.state_json()
        s = json.loads(state_json)
        self._prev_state_cache = getattr(self, "_state_cache", s)
        self._state_cache = s
        return _build_obs_from_state(s)

    def _nearest_other_airport_for_plane0(self) -> Optional[int]:
        s = self._state_cache
        airports = s.get("airports", [])
        planes = s.get("planes", [])
        p0 = None
        for p in planes:
            if int(p.get("id", -1)) == 0:
                p0 = p
                break
        if p0 is None:
            return None
        px, py = float(p0.get("x", 0.0)), float(p0.get("y", 0.0))
        best = None
        best_d2 = 1e30
        for a in airports:
            ax, ay = float(a.get("x", 0.0)), float(a.get("y", 0.0))
            if abs(ax - px) < 1e-6 and abs(ay - py) < 1e-6:
                continue
            d2 = (ax - px) ** 2 + (ay - py) ** 2
            if d2 < best_d2:
                best_d2 = d2
                best = int(a.get("id", 0))
        return best

    def _current_airport_id_for_plane(self, plane_id: int) -> Optional[int]:
        s = self._state_cache
        airports = s.get("airports", [])
        planes = s.get("planes", [])
        p = None
        for pp in planes:
            if int(pp.get("id", -1)) == int(plane_id):
                p = pp
                break
        if p is None:
            return None
        px, py = float(p.get("x", 0.0)), float(p.get("y", 0.0))
        for a in airports:
            ax, ay = float(a.get("x", 0.0)), float(a.get("y", 0.0))
            if abs(ax - px) < 1e-6 and abs(ay - py) < 1e-6:
                return int(a.get("id", 0))
        return None


class RustyRunwaysGymVectorEnv(gym.vector.VectorEnv):
    """Vectorized Gym env backed by the Rust ``VectorGameEnv``.

    Parameters
    ----------
    n_envs : int
        Number of parallel environments.
    seed, num_airports, cash, config_path : optional
        Passed through to ``VectorGameEnv``.
    max_hours : int, default=1000
        Episode horizon (truncation) applied per env.

    Spaces
    ------
    action_space : MultiDiscrete([6, 16, 64, 256])
        (op, plane, selector, dest_index) per env.
    observation_space : Box(float32, shape=(14,))
        Same features as the single-env wrapper.
    """

    N_OPS = 6  # ADVANCE, REFUEL, UNLOAD_ALL, MAINTENANCE, DEPART_BY_INDEX, LOAD_ORDER
    MAX_PLANES = 16
    MAX_SELECT = 64

    def __init__(
        self,
        n_envs: int,
        *,
        seed: Optional[int] = None,
        num_airports: Optional[int] = None,
        cash: Optional[float] = None,
        config_path: Optional[str] = None,
        max_hours: int = 1000,
    ) -> None:
        self.n_envs = int(n_envs)
        self._params = dict(
            seed=seed,
            num_airports=num_airports,
            cash=cash,
            config_path=config_path,
        )
        # infer spaces from single env
        tmp = RustyRunwaysGymEnv(**self._params)
        obs_space = tmp.observation_space
        self.MAX_AIRPORTS = 256
        act_space = spaces.MultiDiscrete(
            [self.N_OPS, self.MAX_PLANES, self.MAX_SELECT, self.MAX_AIRPORTS]
        )
        tmp.close()
        # Some Gymnasium versions have a VectorEnv.__init__ signature that may not accept
        # (num_envs, obs_space, act_space) here. To keep compatibility across versions,
        # avoid passing args and set spaces/num_envs directly.
        try:
            super().__init__(self.n_envs, obs_space, act_space)  # type: ignore[misc]
        except TypeError:
            super().__init__()  # fallback for versions where base __init__ has no params
            self.observation_space = obs_space
            self.action_space = act_space
        # Ensure attribute exists regardless of base handling
        if not hasattr(self, "num_envs"):
            self.num_envs = self.n_envs  # type: ignore[assignment]

        # create vector backend
        self._venv = VectorGameEnv(self.n_envs, **self._params)  # type: ignore[arg-type]
        self._elapsed = np.zeros((self.n_envs,), dtype=np.int64)
        self._max_hours = int(max_hours)
        # last cash per env for reward shaping
        states = [json.loads(s) for s in self._venv.state_all_json()]
        self._last_cash = np.array([float(s.get("cash", 0.0)) for s in states], dtype=np.float32)
        self._last_states = states
        self._actions = None

    def reset(self, *, seed: Optional[int] = None, options: Optional[dict] = None):
        if options is None:
            options = {}
        if seed is not None:
            self._venv.reset_all(seed=seed)
        else:
            self._venv.reset_all()
        self._elapsed[:] = 0
        states = [json.loads(s) for s in self._venv.state_all_json()]
        self._last_states = states
        self._last_cash = np.array([float(s.get("cash", 0.0)) for s in states], dtype=np.float32)
        obs = np.stack([self._obs_from(s) for s in states], axis=0)
        return obs, {}

    def step_async(self, actions) -> None:
        self._actions = np.asarray(actions).astype(int)

    # Some Gymnasium versions require subclasses to implement `step` directly.
    # Delegate to the async/wait pair to satisfy both styles.
    def step(self, actions):
        self.step_async(actions)
        return self.step_wait()

    def step_wait(self):
        acts = np.asarray(self._actions).astype(int)
        if acts.ndim == 1:
            acts = acts.reshape(self.n_envs, -1)
        # Build command list per env
        cmds: List[Optional[str]] = [None] * self.n_envs
        load_requests: List[Optional[Tuple[int, int]]] = [None] * self.n_envs
        for i in range(self.n_envs):
            op, plane, sel, dest_idx = (
                int(acts[i, 0]),
                int(acts[i, 1]),
                int(acts[i, 2]),
                int(acts[i, 3]),
            )
            if op == 0:
                cmds[i] = None
            elif op == 1:
                cmds[i] = f"REFUEL PLANE {plane}"
            elif op == 2:
                cmds[i] = f"UNLOAD ALL FROM {plane}"
            elif op == 3:
                cmds[i] = f"MAINTENANCE {plane}"
            elif op == 4:
                # depart to selected airport id from global list (excluding current)
                s = self._last_states[i]
                ids = self._venv.airport_ids_all()[i]
                curr_id = self._current_airport_id_for_plane(s, plane)
                ids = [x for x in ids if x != curr_id]
                dest = ids[dest_idx % len(ids)] if ids else None
                cmds[i] = f"DEPART PLANE {plane} {dest}" if dest is not None else None
            elif op == 5:
                load_requests[i] = (plane, sel)

        # Resolve load requests
        for i, req in enumerate(load_requests):
            if req is None:
                continue
            plane, sel = req
            ids_lists = self._venv.orders_at_plane_all(plane)
            ids = ids_lists[i]
            if ids:
                order_id = ids[sel % len(ids)]
                cmds[i] = f"LOAD ORDER {order_id} ON {plane}"

        # Execute commands and advance 1h
        self._venv.execute_all(cmds)
        self._venv.step_all(1)
        self._elapsed += 1

        # Get observations and rewards
        states = [json.loads(s) for s in self._venv.state_all_json()]
        obs = np.stack([self._obs_from(s) for s in states], axis=0)
        cash = np.array([float(s.get("cash", 0.0)) for s in states], dtype=np.float32)
        rewards = (cash - self._last_cash).astype(np.float32)
        self._last_cash = cash
        self._last_states = states

        terminated = np.zeros((self.n_envs,), dtype=bool)
        truncated = self._elapsed >= self._max_hours
        infos = [{} for _ in range(self.n_envs)]
        return obs, rewards, terminated, truncated, infos

    def close(self):
        return None

    # helpers
    def _obs_from(self, s: dict) -> np.ndarray:
        # Reuse the same feature extraction as the single-env wrapper
        # without calling its _observe (which pulls fresh state from _env).
        return _build_obs_from_state(s)

    def _proxy_single(self, s: dict):
        """Create a lightweight proxy to reuse single-env observation helpers.

        Parameters
        ----------
        s : dict
            JSON-decoded state dictionary for one environment.

        Returns
        -------
        object
            Proxy object exposing ``_state_cache`` and ``_prev_state_cache`` attributes.
        """

        class _P:
            pass

        p = _P()
        p._env = None  # unused
        p._prev_state_cache = getattr(self, "_prev_state_cache", s)
        p._state_cache = s
        return p

    def _current_airport_id_for_plane(self, s: dict, plane_id: int) -> Optional[int]:
        """Determine the airport id where a plane is currently located.

        Parameters
        ----------
        s : dict
            JSON-decoded state dictionary for one environment.
        plane_id : int
            Identifier of the plane.

        Returns
        -------
        int or None
            Airport id if the plane is parked at a known airport, otherwise ``None``.
        """

        airports = s.get("airports", [])
        planes = s.get("planes", [])
        p = None
        for pp in planes:
            if int(pp.get("id", -1)) == plane_id:
                p = pp
                break
        if p is None:
            return None
        px, py = float(p.get("x", 0.0)), float(p.get("y", 0.0))
        for a in airports:
            ax, ay = float(a.get("x", 0.0)), float(a.get("y", 0.0))
            if abs(ax - px) < 1e-6 and abs(ay - py) < 1e-6:
                return int(a.get("id", 0))
        return None


def make_sb3_envs(
    n_envs: int, seed: Optional[int] = None, **kwargs: Any
) -> List[Callable[[], gym.Env]]:
    """Return a list of callables for SB3's VecEnv builders.

    Parameters
    ----------
    n_envs : int
        Number of environment factories to create.
    seed : int, optional
        Base seed applied to each environment (incremented per env).
    **kwargs
        Additional keyword arguments forwarded to :class:`RustyRunwaysGymEnv`.

    Returns
    -------
    list of Callable
        Thunks suitable for constructing SB3 vectorized environments.

    Examples
    --------
    >>> from stable_baselines3.common.vec_env import DummyVecEnv
    >>> env = DummyVecEnv(make_sb3_envs(2, seed=1))
    """
    thunks: List[Callable[[], gym.Env]] = []
    base_seed = 0 if seed is None else int(seed)
    for i in range(n_envs):

        def _thunk(i=i):
            return RustyRunwaysGymEnv(seed=base_seed + i, **kwargs)

        thunks.append(_thunk)
    return thunks
