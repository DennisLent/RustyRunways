from rusty_runways_py import PyGame


def main():
    commands = ["ADVANCE 1", "ADVANCE 2"]
    game = PyGame(seed=1)
    for cmd in commands:
        try:
            game.execute(cmd)
        except ValueError as e:
            print("error", e)
    print("final time", game.time())


if __name__ == "__main__":
    main()
