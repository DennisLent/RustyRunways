from rusty_runways_py import VectorGameEnv


def main():
    env = VectorGameEnv(8, seed=1)
    env.step_all(1, parallel=True)
    print("times", env.times())
    states = env.state_all_py()
    print("first state time", states[0]["time"])


if __name__ == "__main__":
    main()
