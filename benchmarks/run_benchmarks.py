#!/usr/bin/env python3
"""Heuristic benchmarking suite for RustyRunways.

This script drives the local Python bindings across different scenarios and seeds, records
progression metrics, and renders comparative plots. Run it from a virtual environment
created with ``benchmarks/setup.sh`` so the bindings and plotting dependencies are
available.
"""

import argparse
import csv
import json
import math
import numbers
import os
import re
import statistics
import tempfile
from pathlib import Path

import matplotlib.pyplot as plt
import yaml
from tabulate import tabulate
from tqdm import tqdm

try:  # pragma: no cover - friendly runtime guard
    from rusty_runways import GameEnv
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "rusty_runways python module not found. Did you run benchmarks/setup.sh?\n"
        f"Original error: {exc}"
    )

OUTPUT_DIR = Path(__file__).resolve().parent / "outputs"
OUTPUT_DIR.mkdir(exist_ok=True, parents=True)

PLANE_UPGRADES = [
    {
        "model": "SparrowLight",
        "price": 200_000.0,
        "min_runway": 407.5,
    },
]


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
    """Greedy hauler that prioritises highest value-per-kilometre orders."""

    def __init__(self):
        self.flight_log = FlightLog()
        self.first_upgrade_hour = None
        self.last_purchase_hour = None
        self.max_fleet = 2

    def act(self, env, horizon):
        """Advance the environment by applying a heuristic policy."""

        state = _full_state(env)
        player = state["player"]
        time_now = float(state["time"])
        if self.first_upgrade_hour is None and player.get("fleet_size", 1) > 1:
            self.first_upgrade_hour = time_now

        if self._maybe_buy_plane(env, state):
            return

        airplanes = sorted(state.get("airplanes", []), key=lambda p: int(p.get("id", 0)))

        for plane in airplanes:
            if self._handle_parked_plane(env, state, plane):
                return

        for plane in airplanes:
            if self._handle_nonparked_plane(env, plane, horizon):
                return

        env.step(1)

    def _handle_parked_plane(self, env, state, plane):
        status = str(plane.get("status", "Parked")).lower()
        if status != "parked":
            return False

        plane_id = int(plane.get("id", 0))
        manifest = plane.get("manifest", [])
        if manifest:
            _safe_execute(env, f"UNLOAD ALL FROM {plane_id}")
            env.step(1)
            return True

        px, py = _plane_location(plane)
        airport = _airport_at_coordinate(state, px, py)
        if airport is None:
            return False

        orders = airport.get("orders", [])
        if not orders:
            return False

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
            return False

        order, distance = max(
            candidates,
            key=lambda pair: float(pair[0].get("value", 0.0)) / max(pair[1], 1.0),
        )

        if distance > current_range * 0.9:
            _safe_execute(env, f"REFUEL PLANE {plane_id}")
            env.step(1)
            return True

        _safe_execute(env, f"LOAD ORDER {order['id']} ON {plane_id}")
        env.step(1)
        self.flight_log.record(distance)
        _safe_execute(env, f"DEPART PLANE {plane_id} {order['destination_id']}")
        return True

    def _handle_nonparked_plane(self, env, plane, horizon):
        status = str(plane.get("status", "")).lower()
        plane_id = int(plane.get("id", 0))

        if status in {"loading", "unloading", "refueling", "maintenance"}:
            env.step(1)
            return True

        if status == "broken":
            _safe_execute(env, f"MAINTENANCE {plane_id}")
            env.step(8)
            return True

        if status == "intransit":
            remaining = float(plane.get("hours_remaining") or 1.0)
            max_allowed = max(horizon - env.time(), 1)
            step_hours = min(int(math.ceil(remaining)), max_allowed)
            env.step(max(step_hours, 1))
            return True

        return False

    def _maybe_buy_plane(self, env, state):
        if len(state.get("airplanes", [])) >= self.max_fleet:
            return False

        player = state.get("player", {})
        cash = float(player.get("cash", 0.0))
        time_now = float(state.get("time", 0.0))
        if self.last_purchase_hour is not None and time_now - self.last_purchase_hour < 6:
            return False

        base_plane = state.get("airplanes", [{}])[0]
        base_status = str(base_plane.get("status", "")).lower()
        if base_status != "parked":
            return False
        px, py = _plane_location(base_plane)
        airport = _airport_at_coordinate(state, px, py)
        if airport is None:
            return False

        runway_length = float(
            airport.get("runway_length")
            or airport.get("runway_length_m")
            or airport.get("runway_length_meters")
            or 0.0
        )
        airport_id = airport.get("id")
        if airport_id is None:
            return False

        for option in PLANE_UPGRADES:
            if runway_length >= option["min_runway"] and cash >= option["price"] * 1.1:
                _safe_execute(env, f"BUY PLANE {option['model']} {airport_id}")
                self.last_purchase_hour = time_now
                return True

        return False


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


def build_config_dict(seed, world_params):
    """Construct a world configuration dictionary for a scenario."""

    params = dict(world_params or {})
    version = params.pop("version", 1)
    starting_cash = params.pop("starting_cash", params.pop("cash", 1_000_000.0))

    config = {"version": version, "seed": seed, "starting_cash": float(starting_cash)}
    for key, value in params.items():
        config[key] = value
    return config


def run_seed(seed, hours, scenario_params):
    """Simulate a single seed and collect metrics.

    Parameters
    ----------
    seed : int
        Random seed used for world generation.
    hours : int
        Horizon in in-game hours.
    scenario_params : dict
        World overrides and metadata for the scenario variant.

    Returns
    -------
    dict
        Summary metrics for the run.
    list of dict
        Time-series snapshots containing cash, fleet size, and deliveries.
    """

    tmp_path = None
    config_path = scenario_params.get("config_path")
    world = scenario_params.get("world") or {}

    if config_path:
        env = GameEnv(config_path=config_path)
    else:
        cfg = build_config_dict(seed, world)
        if "num_airports" not in cfg and "airports" not in cfg:
            raise ValueError("scenario must define either num_airports or airports")
        tmp = tempfile.NamedTemporaryFile("w", suffix=".yaml", delete=False)
        yaml.safe_dump(cfg, tmp)
        tmp_path = tmp.name
        tmp.close()
        env = GameEnv(config_path=tmp_path)

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
        initial_cash = timeline[0]["cash"] if timeline else 0.0
        while env.time() < hours:
            agent.act(env, hours)
            record_snapshot()
            if env.time() >= hours:
                break
            env.step(1)
            record_snapshot()

        state_final = _full_state(env)
        total_cash = float(state_final["player"].get("cash", initial_cash))
        cash_gain = total_cash - initial_cash
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
        if tmp_path:
            os.unlink(tmp_path)


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
        "avg_cash_gain": statistics.mean(r["cash_gain"] for r in results),
        "avg_upgrade_hour": statistics.mean(upgrade_samples) if upgrade_samples else float("nan"),
        "avg_route_len": statistics.mean(r["avg_route_len"] for r in results if r["avg_route_len"] > 0)
        if any(r["avg_route_len"] > 0 for r in results)
        else 0.0,
    }


def write_time_series(seed, timeline, out_dir):
    """Persist the timeline for a seed to CSV.

    Parameters
    ----------
    seed : int
        Seed identifier used in the simulation.
    timeline : list of dict
        Time-series snapshots captured during :func:`run_seed`.
    out_dir : pathlib.Path
        Directory where the CSV should be written.
    """

    path = out_dir / f"seed_{seed}_timeline.csv"
    with path.open("w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=["time", "cash", "fleet_size", "deliveries"])
        writer.writeheader()
        writer.writerows(timeline)


def plot_series(series_by_seed, out_dir, scenario_name):
    """Generate comparative plots for cash, fleet size, and deliveries.

    Parameters
    ----------
    series_by_seed : dict
        Mapping from seed to the timeline produced by :func:`run_seed`.
    out_dir : pathlib.Path
        Output directory for the PNG plots.
    scenario_name : str
        Name of the scenario (used in plot titles).
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
        plt.title(f"{scenario_name}: {ylabel}")
        plt.legend()
        plt.tight_layout()
        plt.savefig(out_dir / filename)
        plt.close()


def plot_cash_distribution(results, out_dir, scenario_name):
    """Plot margin-per-hour distribution across seeds.

    Parameters
    ----------
    results : list of dict
        Metrics returned by :func:`run_seed` for each seed.
    out_dir : pathlib.Path
        Directory where the bar chart should be written.
    scenario_name : str
        Scenario label for the plot title.
    """

    plt.figure(figsize=(6, 4))
    margins = [r["margin_per_hour"] for r in results]
    plt.bar([str(r["seed"]) for r in results], margins, color="#1976d2")
    plt.xlabel("Seed")
    plt.ylabel("$ per hour")
    plt.title(f"{scenario_name}: Margin per Hour")
    plt.tight_layout()
    plt.savefig(out_dir / "margin_per_hour.png")
    plt.close()


def write_summary_csv(records, out_dir):
    """Persist aggregated scenario metrics to CSV."""

    if not records:
        return

    path = out_dir / "scenario_summary.csv"
    fieldnames = [
        "scenario_name",
        "base_name",
        "variant_label",
        "variant_value",
        "sweep_parameter",
        "sweep_value",
        "parameter_value",
        "hours",
        "seed_count",
        "avg_margin_per_hour",
        "avg_cash_gain",
        "avg_feasible_ratio",
        "avg_upgrade_hour",
        "avg_route_len",
        "metadata_json",
    ]
    with path.open("w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        for record in records:
            row = {key: record.get(key) for key in fieldnames}
            row["metadata_json"] = json.dumps(record.get("metadata", {}), sort_keys=True)
            writer.writerow(row)


def plot_parameter_sweeps(records, out_dir):
    """Render plots for parameter sweeps across scenario variants."""

    if not records:
        return

    summary_dir = out_dir / "summary"
    summary_dir.mkdir(parents=True, exist_ok=True)

    metrics = {
        "avg_margin_per_hour": ("Margin per Hour ($/hr)", "margin"),
        "avg_cash_gain": ("Cash Gain ($)", "cash"),
        "avg_feasible_ratio": ("Feasible Orders Ratio", "feasible"),
    }

    base_records = {}
    grouped = {}
    for record in records:
        if record.get("variant_label") is None:
            base_records.setdefault(record.get("base_name"), record)
        parameter = record.get("sweep_parameter")
        if parameter:
            grouped.setdefault((record.get("base_name"), parameter), []).append(record)

    for (base_name, parameter), items in list(grouped.items()):
        base_record = base_records.get(base_name)
        if base_record and all(item is not base_record for item in items):
            value = get_nested_value(base_record.get("world") or {}, parameter)
            if value is None:
                value = base_record.get("parameter_value")
            if value is not None:
                baseline = dict(base_record)
                baseline["sweep_parameter"] = parameter
                baseline["sweep_value"] = value
                baseline["parameter_value"] = value
                baseline["variant_label"] = baseline.get("variant_label") or "baseline"
                items.append(baseline)

        numeric_entries = []
        categorical_entries = []
        for entry in items:
            axis_value = entry.get("sweep_value")
            if axis_value is None:
                axis_value = entry.get("parameter_value")
            if isinstance(axis_value, numbers.Real) and not math.isnan(float(axis_value)):
                numeric_entries.append((float(axis_value), entry))
            else:
                label = entry.get("variant_label")
                if not label:
                    label = str(axis_value)
                categorical_entries.append((label, entry))

        for metric_key, (ylabel, suffix) in metrics.items():
            if numeric_entries and len(numeric_entries) >= 2:
                numeric_entries.sort(key=lambda pair: pair[0])
                xs = [pair[0] for pair in numeric_entries]
                ys = [pair[1].get(metric_key) for pair in numeric_entries]
                if any(value is None for value in ys):
                    continue
                plt.figure(figsize=(6, 4))
                plt.plot(xs, ys, marker="o")
                plt.xlabel(parameter)
                plt.ylabel(ylabel)
                plt.title(f"{base_name}: {ylabel} vs {parameter}")
                plt.tight_layout()
                file_slug = f"{slugify_label(base_name)}__{slugify_label(parameter)}__{suffix}.png"
                plt.savefig(summary_dir / file_slug)
                plt.close()
            elif categorical_entries:
                labels = [label for label, _ in categorical_entries]
                values = [entry.get(metric_key) for _, entry in categorical_entries]
                if any(value is None for value in values):
                    continue
                plt.figure(figsize=(6, 4))
                plt.bar(labels, values, color="#1976d2")
                plt.xlabel(parameter)
                plt.ylabel(ylabel)
                plt.title(f"{base_name}: {ylabel} vs {parameter}")
                plt.tight_layout()
                file_slug = f"{slugify_label(base_name)}__{slugify_label(parameter)}__{suffix}_categorical.png"
                plt.savefig(summary_dir / file_slug)
                plt.close()
def deep_merge(base, override):
    """Recursively merge two dictionaries (override wins)."""

    result = dict(base)
    for key, value in override.items():
        if isinstance(value, dict) and isinstance(result.get(key), dict):
            result[key] = deep_merge(result[key], value)
        else:
            result[key] = value
    return result


def ensure_list(value):
    """Normalize a scalar or list-like value to a list."""

    if value is None:
        return []
    if isinstance(value, (list, tuple)):
        return list(value)
    return [value]


def slugify_label(label):
    """Create a filesystem-friendly slug from a label."""

    text = str(label).strip()
    text = re.sub(r"[^0-9A-Za-z._-]+", "-", text)
    text = re.sub(r"-{2,}", "-", text)
    return text.strip("-") or "variant"


def nested_override_from_path(path, value):
    """Build a nested dictionary override for a dotted path."""

    parts = str(path).split(".")
    result = {}
    cursor = result
    for idx, key in enumerate(parts):
        if idx == len(parts) - 1:
            cursor[key] = value
        else:
            cursor = cursor.setdefault(key, {})
    return result


def get_nested_value(data, path):
    """Retrieve a value from a nested dictionary using a dotted path."""

    current = data
    for key in str(path).split("."):
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return current


def normalize_variant_entry(entry):
    """Normalize a variant definition into override metadata."""

    if not isinstance(entry, dict):
        raise TypeError("variant entries must be dicts")

    overrides = deep_merge({}, entry.get("overrides", {}))
    value = entry.get("value")
    parameter = entry.get("parameter")
    metadata = dict(entry.get("metadata", {}))
    if parameter is not None:
        overrides = deep_merge(overrides, nested_override_from_path(parameter, value))
        metadata.setdefault("sweep_parameter", parameter)
        metadata.setdefault("sweep_value", value)
        metadata.setdefault("parameter_value", value)

    return {
        "label": entry.get("label") or entry.get("name"),
        "overrides": overrides,
        "value": value,
        "metadata": metadata,
        "config_path": entry.get("config_path"),
        "seeds": entry.get("seeds"),
        "hours": entry.get("hours"),
    }


def build_variants_from_sweep(entry):
    """Expand a sweep definition into variant entries."""

    if not isinstance(entry, dict):
        raise TypeError("sweep definitions must be dicts")

    parameter = entry.get("parameter")
    if not parameter:
        raise ValueError("sweep definitions require a 'parameter' field")

    label_prefix = entry.get("label")
    base_metadata = dict(entry.get("metadata", {}))
    base_overrides = deep_merge({}, entry.get("overrides", {}))
    values = ensure_list(entry.get("values"))
    if not values:
        raise ValueError("sweep definitions require non-empty 'values'")

    variants = []
    for raw in values:
        if isinstance(raw, dict):
            value = raw.get("value")
            label = raw.get("label")
            extra_overrides = raw.get("overrides", {})
            extra_metadata = raw.get("metadata", {})
            config_path = raw.get("config_path")
            seeds = raw.get("seeds")
            hours = raw.get("hours")
        else:
            value = raw
            label = None
            extra_overrides = {}
            extra_metadata = {}
            config_path = None
            seeds = None
            hours = None

        overrides = deep_merge(base_overrides, nested_override_from_path(parameter, value))
        if extra_overrides:
            overrides = deep_merge(overrides, extra_overrides)

        metadata = dict(base_metadata)
        metadata.setdefault("sweep_parameter", parameter)
        metadata.setdefault("sweep_value", value)
        metadata.setdefault("parameter_value", value)
        metadata.update(extra_metadata)

        if label is None and label_prefix:
            if value is None:
                label = label_prefix
            else:
                label = f"{label_prefix}={value}"

        variants.append(
            {
                "label": label,
                "overrides": overrides,
                "value": value,
                "metadata": metadata,
                "config_path": config_path,
                "seeds": seeds,
                "hours": hours,
            }
        )

    return variants


def load_scenarios(path, cli_defaults):
    """Load scenario definitions from a YAML configuration file."""

    with open(path, "r", encoding="utf-8") as fh:
        data = yaml.safe_load(fh) or {}

    defaults = data.get("defaults", {})
    scenarios = data.get("scenarios", [])
    if not scenarios:
        raise ValueError("scenario config must define at least one scenario")

    runs = []
    for idx, entry in enumerate(scenarios):
        merged = deep_merge(defaults, entry)

        name = merged.pop("name", None) or f"scenario_{idx}"
        seeds = ensure_list(merged.pop("seeds", cli_defaults["seeds"]))
        hours = int(merged.pop("hours", cli_defaults["hours"]))
        config_path = merged.pop("config_path", None)
        include_base = bool(merged.pop("include_base", True))
        metadata = dict(merged.pop("metadata", {}))

        variants = merged.pop("variants", None)
        sweep = merged.pop("sweep", None) or merged.pop("sweeps", None)

        if config_path is None:
            if "cash" not in merged and "starting_cash" not in merged:
                merged["cash"] = cli_defaults["cash"]
            if "num_airports" not in merged and "airports" not in merged:
                merged["num_airports"] = cli_defaults["airports"]

        merged.setdefault("version", 1)

        variant_specs = []
        if include_base:
            variant_specs.append({
                "label": None,
                "overrides": {},
                "metadata": {},
                "value": None,
                "config_path": None,
                "seeds": None,
                "hours": None,
            })

        if sweep:
            sweep_entries = sweep if isinstance(sweep, list) else [sweep]
            for sweep_entry in sweep_entries:
                variant_specs.extend(build_variants_from_sweep(sweep_entry))

        if variants:
            for variant in variants:
                variant_specs.append(normalize_variant_entry(variant))

        if not variant_specs:
            variant_specs.append({
                "label": None,
                "overrides": {},
                "metadata": {},
                "value": None,
                "config_path": None,
                "seeds": None,
                "hours": None,
            })

        for vidx, spec in enumerate(variant_specs):
            overrides = spec.get("overrides") or {}
            world = deep_merge(merged, overrides)
            variant_label = spec.get("label")
            slug = slugify_label(variant_label) if variant_label else ""
            unique_name = name if not variant_label else f"{name}__{slug}"
            if not variant_label and vidx > 0:
                unique_name = f"{name}__v{vidx}"

            run_metadata = dict(metadata)
            run_metadata.update(spec.get("metadata", {}))

            run_seeds = ensure_list(spec.get("seeds")) if spec.get("seeds") is not None else seeds
            run_hours = int(spec.get("hours")) if spec.get("hours") is not None else hours

            runs.append(
                {
                    "name": unique_name,
                    "base_name": name,
                    "variant_label": variant_label,
                    "variant_value": spec.get("value"),
                    "seeds": run_seeds,
                    "hours": run_hours,
                    "world": world,
                    "config_path": spec.get("config_path") or config_path,
                    "metadata": run_metadata,
                }
            )

    return runs


def main():  # pragma: no cover - CLI dispatch
    parser = argparse.ArgumentParser(description="Run RustyRunways heuristic benchmarks")
    parser.add_argument("--seeds", nargs="*", type=int, default=[0, 1, 2], help="Seeds to evaluate")
    parser.add_argument("--hours", type=int, default=120, help="Simulation horizon (hours)")
    parser.add_argument("--airports", type=int, default=6, help="Number of airports to generate")
    parser.add_argument("--cash", type=float, default=1_000_000.0, help="Starting cash")
    parser.add_argument("--scenario-config", type=str, help="Path to scenario YAML definition")
    args = parser.parse_args()

    cli_defaults = {
        "seeds": args.seeds,
        "hours": args.hours,
        "airports": args.airports,
        "cash": args.cash,
    }

    if args.scenario_config:
        scenarios = load_scenarios(args.scenario_config, cli_defaults)
    else:
        scenarios = [
            {
                "name": "cli",
                "base_name": "cli",
                "variant_label": None,
                "variant_value": None,
                "seeds": list(args.seeds),
                "hours": args.hours,
                "world": {"cash": args.cash, "num_airports": args.airports, "version": 1},
                "config_path": None,
                "metadata": {},
            }
        ]

    summary_records = []

    for scenario in scenarios:
        scenario_dir = OUTPUT_DIR / scenario["name"]
        scenario_dir.mkdir(parents=True, exist_ok=True)
        scenario_results = []
        scenario_series = {}

        for seed in tqdm(scenario["seeds"], desc=f"{scenario['name']} seeds"):
            metrics, timeline = run_seed(seed, scenario["hours"], scenario)
            scenario_results.append(metrics)
            scenario_series[seed] = timeline
            write_time_series(seed, timeline, scenario_dir)

        headers, rows = summarize(scenario_results)
        print(f"\nScenario: {scenario['name']}")
        print(tabulate(rows, headers=headers, tablefmt="github"))
        agg = aggregate(scenario_results)
        if agg:
            print("  Aggregates:")
            for key, value in agg.items():
                print(f"    {key}: {value:.3f}")

        plot_series(scenario_series, scenario_dir, scenario["name"])
        plot_cash_distribution(scenario_results, scenario_dir, scenario["name"])

        metadata = dict(scenario.get("metadata", {}))
        summary_record = {
            "scenario_name": scenario["name"],
            "base_name": scenario.get("base_name", scenario["name"]),
            "variant_label": scenario.get("variant_label"),
            "variant_value": scenario.get("variant_value"),
            "sweep_parameter": metadata.get("sweep_parameter"),
            "sweep_value": metadata.get("sweep_value"),
            "parameter_value": metadata.get("parameter_value"),
            "hours": scenario["hours"],
            "seed_count": len(scenario["seeds"]),
            "metadata": metadata,
            "world": scenario.get("world"),
        }
        summary_record.update(agg)

        parameter = summary_record.get("sweep_parameter")
        if parameter and summary_record.get("parameter_value") is None:
            summary_record["parameter_value"] = get_nested_value(scenario.get("world") or {}, parameter)

        summary_records.append(summary_record)

    write_summary_csv(summary_records, OUTPUT_DIR)
    plot_parameter_sweeps(summary_records, OUTPUT_DIR)


if __name__ == "__main__":  # pragma: no cover
    main()
