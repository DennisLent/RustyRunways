from rusty_runways_py import PyGame

SCRIPT = [
    "SHOW CASH",
    "ADVANCE 1",
]

if __name__ == "__main__":
    g = PyGame(seed=1, num_airports=5, cash=1_000_000)
    for line in SCRIPT:
        g.execute(line)
        print(g.drain_log())
