#!/usr/bin/env python3
"""Heuristic benchmarking suite for RustyRunways.

This script builds on the local Python bindings to simulate multiple seeds, collect
engagement metrics, and generate comparative plots. Run with the virtual
environment prepared via ``benchmarks/setup.sh``.
"""

from __future__ import annotations

import argparse
import csv
import json
import math
import statistics
from collections import defaultdict
from pathlib import Path

from tabulate import tabulate
from tqdm import tqdm
import matplotlib.pyplot as plt

try:  # pragma: no cover - friendly runtime guard
    from rusty_runways import GameEnv
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "rusty_runways python module not found. Did you run benchmarks/setup.sh?\n"
        f"Original error: {exc}"
    )

OUTPUT_DIR = Path(__file__).resolve().parent / "outputs"
OUTPUT_DIR.mkdir(exist_ok=True, parents=True)


class FlightLog:
    """Record of flown routes.

    Attributes
    ----------
    distances : list of float
        Route distances recorded after each departure.
    """

    def __init__(self):
        self.distances = []

    def record(self, distance):
        """Append a traversed route to the log.

        Parameters
        ----------
        distance : float
            Route length in kilometres.
        """

        if distance > 0.0:
            self.distances.append(distance)

    def average(self):
        """Return the mean route distance."""

        return float(statistics.mean(self.distances)) if self.distances else 0.0


class HeuristicAgent:
    """Greedy hauler that always flies the best value-to-distance order."""

    def __init__(self):
        self.flight_log = FlightLog()
        self.first_upgrade_hour = None

    def act(self, env, horizon):
        """Advance the environment by applying a heuristic policy.

        Parameters
        ----------
        env : rusty_runways.GameEnv
            Environment instance.
        horizon : int
            Maximum simulation horizon in hours.
        """

        state = _full_state(env)
        time_now = state["time"]
        player = state["player"]
        if self.first_upgrade_hour is None and player.get("fleet_size", 1) > 1:
            self.first_upgrade_hour = float(time_now)

        plane = state["airplanes"][0]
        status = str(plane.get("status", "Parked")).lower()

        if status == "intransit":
            remaining = plane.get("hours_remaining") or 1
            env.step(min(int(remaining), max(horizon - env.time(), 1)))
            return

        if status in {"loading", "unloading", "refueling", "maintenance"}:
            env.step(1)
            return

        if status == "broken":
            _safe_execute(env, "MAINTENANCE 0")
            env.step(8)
            return

        if status != "parked":
            env.step(1)
            return

        manifest = plane.get("manifest", [])
        if manifest:
            _safe_execute(env, "UNLOAD ALL FROM 0")
            env.step(1)
            return

        px, py = _plane_location(plane)
        airport = _airport_at_coordinate(state, px, py)
        if airport is None:
            env.step(1)
            return

        orders = airport.get("orders", [])
        if not orders:
            env.step(1)
            return

        specs = plane.get("specs", {})
        payload_cap = float(specs.get("payload_capacity", 0.0))
        payload_current = float(plane.get("current_payload", 0.0))
        fuel_cap = float(specs.get("fuel_capacity", 0.0))
        fuel_consumption = float(specs.get("fuel_consumption", 1.0))
        cruise_speed = float(specs.get("cruise_speed", 1.0))
        current_fuel = float(plane.get("current_fuel", fuel_cap))
        max_range = (fuel_cap / fuel_consumption) * cruise_speed if fuel_consumption else 0.0
        current_range = (
            (current_fuel / fuel_consumption) * cruise_speed if fuel_consumption else 0.0
        )

        candidates = []
        for order in orders:
            weight = float(order.get("weight", 0.0))
            if weight + payload_current > payload_cap:
                continue
            dest_id = order.get("destination_id")
            dest_coord = _airport_coordinate(state, dest_id)
            if dest_coord is None:
                continue
            distance = _distance((px, py), dest_coord)
            if distance > max_range:
                continue
            candidates.append((order, distance))

        if not candidates:
            env.step(1)
            return

        order, distance = max(
            candidates,
            key=lambda pair: float(pair[0].get("value", 0.0)) / max(pair[1], 1.0),
        )

        if distance > current_range * 0.9:
            _safe_execute(env, "REFUEL PLANE 0")
            env.step(1)
            return

        _safe_execute(env, f"LOAD ORDER {order['id']} ON 0")
        env.step(1)
        self.flight_log.record(distance)
        _safe_execute(env, f"DEPART PLANE 0 {order['destination_id']}")


def _safe_execute(env, command):
    """Execute a command, ignoring recoverable errors."""

    try:
        env.execute(command)
    except Exception:
        pass


def _full_state(env):
    """Return the full game state as a Python dictionary."""

    return json.loads(env.full_state_json())


def _distance(a, b):
    """Compute Euclidean distance between two points."""

    ax, ay = a
    bx, by = b
    return math.hypot(bx - ax, by - ay)


def _plane_location(plane):
    """Extract the (x, y) location tuple from a plane entry."""

    loc = plane.get("location", {})
    return float(loc.get("x", 0.0)), float(loc.get("y", 0.0))


def _airport_at_coordinate(state, x, y):
    """Locate the airport data structure at the specified coordinate."""

    for airport, coord in state["map"]["airports"]:
        if abs(coord["x"] - x) < 1e-6 and abs(coord["y"] - y) < 1e-6:
            return airport
    return None


def _airport_coordinate(state, airport_id):
    """Return the coordinate tuple for an airport id."""

    if airport_id is None:
        return None
    for airport, coord in state["map"]["airports"]:
        if int(airport["id"]) == int(airport_id):
            return coord["x"], coord["y"]
    return None


def initial_feasibility(state):
    """Count initial orders that are feasible for the starter plane."""

    plane = state["airplanes"][0]
    px, py = _plane_location(plane)
    airport = _airport_at_coordinate(state, px, py)
    if airport is None:
        return 0, 0
    orders = airport.get("orders", [])
    specs = plane.get("specs", {})
    payload_cap = float(specs.get("payload_capacity", 0.0))
    fuel_cap = float(specs.get("fuel_capacity", 0.0))
    fuel_consumption = float(specs.get("fuel_consumption", 1.0))
    cruise_speed = float(specs.get("cruise_speed", 1.0))
    max_range = (fuel_cap / fuel_consumption) * cruise_speed if fuel_consumption else 0.0

    feasible = 0
    for order in orders:
        weight = float(order.get("weight", 0.0))
        dest_coord = _airport_coordinate(state, order.get("destination_id"))
        if dest_coord is None:
            continue
        distance = _distance((px, py), dest_coord)
        if weight <= payload_cap and distance <= max_range:
            feasible += 1
    return feasible, len(orders)


def run_seed(seed, hours, num_airports, starting_cash):
    """Simulate a single seed and collect metrics.

    Parameters
    ----------
    seed : int
        Random seed used for world generation.
    hours : int
        Horizon in in-game hours.
    num_airports : int
        Number of airports to generate when not using a config file.
    starting_cash : float
        Initial cash for the simulation.

    Returns
    -------
    dict
        Summary metrics for the run.
    list of dict
        Time-series snapshots containing cash, fleet size, and deliveries.
    """

    env = GameEnv(seed=seed, num_airports=num_airports, cash=starting_cash)
    try:
        state0 = _full_state(env)
        feasible, visible = initial_feasibility(state0)
        agent = HeuristicAgent()
        timeline = []

        def record_snapshot():
            snap = _full_state(env)
            player = snap["player"]
            timeline.append(
                {
                    "time": snap["time"],
                    "cash": float(player.get("cash", 0.0)),
                    "fleet_size": float(player.get("fleet_size", 1)),
                    "deliveries": float(player.get("orders_delivered", 0)),
                }
            )

        record_snapshot()
        while env.time() < hours:
            agent.act(env, hours)
            record_snapshot()
            if env.time() >= hours:
                break
            env.step(1)
            record_snapshot()

        state_final = _full_state(env)
        total_cash = float(state_final["player"].get("cash", starting_cash))
        cash_gain = total_cash - starting_cash
        total_hours = max(env.time(), 1)
        avg_route = agent.flight_log.average()
        metrics = {
            "seed": seed,
            "hours": total_hours,
            "cash_gain": cash_gain,
            "margin_per_hour": cash_gain / total_hours,
            "orders_feasible": feasible,
            "orders_visible": visible,
            "feasible_ratio": (feasible / visible) if visible else 0.0,
            "first_upgrade_hour": agent.first_upgrade_hour,
            "avg_route_len": avg_route,
            "num_routes": len(agent.flight_log.distances),
        }
        return metrics, timeline
    finally:
        del env


def summarize(results):
    """Prepare tabular output for per-seed metrics.

    Parameters
    ----------
    results : list of dict
        Metrics returned by :func:`run_seed` for each seed.

    Returns
    -------
    headers : list of str
        Column headers for ``tabulate``.
    rows : list of list of str
        Row values ready for table rendering.
    """

    rows = []
    for res in results:
        rows.append(
            [
                res["seed"],
                f"{res['feasible_ratio']*100:5.1f}% ({res['orders_feasible']}/{res['orders_visible']})",
                f"{res['margin_per_hour']:7.1f}",
                f"{res['cash_gain']:8.1f}",
                res["first_upgrade_hour"] if res["first_upgrade_hour"] is not None else "-",
                f"{res['avg_route_len']:7.1f}",
                res["num_routes"],
            ]
        )
    headers = [
        "Seed",
        "Feasible Orders",
        "$/hr",
        "Cash Gain",
        "First Upgrade (h)",
        "Avg Route km",
        "Routes",
    ]
    return headers, rows


def aggregate(results):
    """Compute aggregate averages across seeds.

    Parameters
    ----------
    results : list of dict
        Metrics returned by :func:`run_seed` for each seed.

    Returns
    -------
    dict
        Summary statistics keyed by metric name.
    """

    if not results:
        return {}
    upgrade_samples = [r["first_upgrade_hour"] for r in results if r["first_upgrade_hour"] is not None]
    return {
        "avg_feasible_ratio": statistics.mean(r["feasible_ratio"] for r in results),
        "avg_margin_per_hour": statistics.mean(r["margin_per_hour"] for r in results),
        "avg_upgrade_hour": statistics.mean(upgrade_samples) if upgrade_samples else float("nan"),
        "avg_route_len": statistics.mean(r["avg_route_len"] for r in results if r["avg_route_len"] > 0)
        if any(r["avg_route_len"] > 0 for r in results)
        else 0.0,
    }


def write_time_series(seed, timeline):
    """Persist the timeline for a seed to CSV.

    Parameters
    ----------
    seed : int
        Seed identifier used in the simulation.
    timeline : list of dict
        Time-series snapshots captured during :func:`run_seed`.
    """

    path = OUTPUT_DIR / f"seed_{seed}_timeline.csv"
    with path.open("w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=["time", "cash", "fleet_size", "deliveries"])
        writer.writeheader()
        writer.writerows(timeline)


def plot_series(series_by_seed):
    """Generate comparative plots for cash, fleet size, and deliveries.

    Parameters
    ----------
    series_by_seed : dict
        Mapping from seed to the timeline produced by :func:`run_seed`.
    """

    metrics = {
        "cash": ("Cash ($)", "cash_over_time.png"),
        "fleet_size": ("Fleet Size", "fleet_over_time.png"),
        "deliveries": ("Deliveries", "deliveries_over_time.png"),
    }
    for key, (ylabel, filename) in metrics.items():
        plt.figure(figsize=(8, 4))
        for seed, timeline in series_by_seed.items():
            times = [row["time"] for row in timeline]
            values = [row[key] for row in timeline]
            plt.plot(times, values, label=f"Seed {seed}")
        plt.xlabel("Hours")
        plt.ylabel(ylabel)
        plt.title(ylabel)
        plt.legend()
        plt.tight_layout()
        plt.savefig(OUTPUT_DIR / filename)
        plt.close()


def plot_cash_distribution(results):
    """Plot margin-per-hour distribution across seeds.

    Parameters
    ----------
    results : list of dict
        Metrics returned by :func:`run_seed` for each seed.
    """

    plt.figure(figsize=(6, 4))
    margins = [r["margin_per_hour"] for r in results]
    plt.bar([str(r["seed"]) for r in results], margins, color="#1976d2")
    plt.xlabel("Seed")
    plt.ylabel("$ per hour")
    plt.title("Margin per Hour by Seed")
    plt.tight_layout()
    plt.savefig(OUTPUT_DIR / "margin_per_hour.png")
    plt.close()


def main():
    """Entry point for the benchmarking runner."""

    parser = argparse.ArgumentParser(description="Run RustyRunways heuristic benchmarks")
    parser.add_argument("--seeds", nargs="*", type=int, default=[0, 1, 2], help="Seeds to evaluate")
    parser.add_argument("--hours", type=int, default=120, help="Simulation horizon (hours)")
    parser.add_argument("--airports", type=int, default=6, help="Number of airports to generate")
    parser.add_argument("--cash", type=float, default=1_000_000.0, help="Starting cash")
    args = parser.parse_args()

    results = []
    series_by_seed = {}

    for seed in tqdm(args.seeds, desc="Seeds"):
        metrics, timeline = run_seed(seed, args.hours, args.airports, args.cash)
        results.append(metrics)
        series_by_seed[seed] = timeline
        write_time_series(seed, timeline)

    headers, rows = summarize(results)
    print(tabulate(rows, headers=headers, tablefmt="github"))
    agg = aggregate(results)
    if agg:
        print("\nAggregate metrics:")
        for key, value in agg.items():
            print(f"  {key}: {value:.3f}")

    plot_series(series_by_seed)
    plot_cash_distribution(results)


if __name__ == "__main__":  # pragma: no cover - CLI dispatch
    main()
