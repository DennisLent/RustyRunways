from rusty_runways_py import PyGame
import random


def main():
    game = PyGame(seed=1, num_airports=4, cash=650_000.0)
    for _ in range(5):
        game.step(1)
        if random.random() < 0.5:
            try:
                game.execute("ADVANCE 1")
            except ValueError as e:
                print("command error", e)
        print("time", game.time(), "cash", game.cash())
    print(game.state_json())


if __name__ == "__main__":
    main()
