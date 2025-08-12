from rusty_runways_py import PyGame

if __name__ == "__main__":
    g = PyGame(seed=1, num_airports=5, cash=1_000_000)
    for t in range(100):
        g.step(1)
        print(g.time(), g.cash())
