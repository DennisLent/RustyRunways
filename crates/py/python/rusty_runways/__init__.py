from rusty_runways_py import GameEnv, VectorGameEnv

# Optional Gym wrappers: load lazily and give a clear error if Gymnasium is missing.
try:
    from .gym_env import RustyRunwaysGymEnv, RustyRunwaysGymVectorEnv, make_sb3_envs  # type: ignore
except Exception as _e:  # gymnasium not installed or import error inside wrapper
    _MISSING_GYM_MSG = (
        "Gymnasium is required for RustyRunways gym wrappers.\n"
        "Install with: pip install 'rusty-runways[gym]'\n"
        "Or: pip install gymnasium"
    )

    class _MissingGym:  # minimal callable raising a helpful error on use
        def __init__(self, *args, **kwargs):
            raise ImportError(_MISSING_GYM_MSG)

    def make_sb3_envs(*args, **kwargs):  # noqa: D401
        raise ImportError(_MISSING_GYM_MSG)

    RustyRunwaysGymEnv = _MissingGym  # type: ignore
    RustyRunwaysGymVectorEnv = _MissingGym  # type: ignore

__all__ = [
    "RustyRunwaysGymEnv",
    "make_sb3_envs",
    "RustyRunwaysGymVectorEnv",
    "GameEnv",
    "VectorGameEnv",
]
