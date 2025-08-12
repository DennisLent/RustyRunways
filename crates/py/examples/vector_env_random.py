from rusty_runways_py import PyVectorEnv

if __name__ == "__main__":
    env = PyVectorEnv(8, seed=123, num_airports=5, cash=1_000_000)
    env.step_all(1, parallel=True)
    for s in env.states_json():
        print(s)
