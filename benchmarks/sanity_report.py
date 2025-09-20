#!/usr/bin/env python3
"""Generate sanity-check visualisations for the default Rusty Runways balance.

The script optionally runs the benchmark suite for the provided scenario config and
then renders per-seed graphs that highlight feasibility and phase performance metrics.
"""

from __future__ import annotations

import argparse
import csv
import json
import statistics
import subprocess
from pathlib import Path
from typing import Dict, Iterable, List

import matplotlib.pyplot as plt
import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CONFIG = REPO_ROOT / "benchmarks" / "sanity.yaml"
DEFAULT_OUTPUT = REPO_ROOT / "benchmarks" / "outputs"


def _load_config(path: Path) -> List[str]:
    with path.open("r", encoding="utf-8") as handle:
        data = yaml.safe_load(handle)
    scenarios = data.get("scenarios", [])
    if not scenarios:
        raise SystemExit(f"No scenarios defined in {path}")
    return [entry["name"] for entry in scenarios]


def _maybe_run_benchmarks(config: Path) -> None:
    cmd = [
        "python",
        str(REPO_ROOT / "benchmarks" / "run_benchmarks.py"),
        "--scenario-config",
        str(config),
    ]
    subprocess.run(cmd, check=True, cwd=REPO_ROOT)


def _read_seed_metrics(scenario: str, output_root: Path) -> List[Dict[str, float]]:
    metrics_path = output_root / scenario / "seed_metrics.csv"
    if not metrics_path.exists():
        raise SystemExit(f"Seed metrics not found for {scenario}: {metrics_path}")
    rows: List[Dict[str, float]] = []
    with metrics_path.open("r", newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for raw in reader:
            numeric = {}
            for key, value in raw.items():
                if key == "seed":
                    numeric[key] = int(value)
                else:
                    try:
                        numeric[key] = float(value)
                    except ValueError:
                        pass
            rows.append(numeric)
    if not rows:
        raise SystemExit(f"No rows read from {metrics_path}")
    rows.sort(key=lambda r: r["seed"])
    return rows


def _summarise(values: Iterable[float]) -> Dict[str, float]:
    series = list(values)
    return {
        "min": min(series),
        "max": max(series),
        "mean": statistics.fmean(series),
        "median": statistics.median(series),
        "stdev": statistics.pstdev(series),
    }


def _plot_feasibility(rows: List[Dict[str, float]], out_dir: Path) -> None:
    seeds = [row["seed"] for row in rows]
    feasible = [row["feasible_ratio"] for row in rows]

    fig, ax = plt.subplots(figsize=(8, 4.5))
    ax.plot(seeds, feasible, marker="o", linestyle="-", color="#2f7ed8")
    ax.set_title("Feasible Order Ratio by Seed")
    ax.set_xlabel("Seed")
    ax.set_ylabel("Feasible / Visible")
    ax.set_ylim(0, 1.05)
    ax.axhspan(0.55, 0.7, color="#8bc34a", alpha=0.25, label="Target band (55%-70%)")
    ax.grid(True, linestyle=":", alpha=0.4)
    ax.legend(loc="lower right")
    fig.tight_layout()
    fig.savefig(out_dir / "sanity_feasible_ratio.png", dpi=160)
    plt.close(fig)


def _plot_phase_metric(
    rows: List[Dict[str, float]],
    out_dir: Path,
    column_map: Dict[str, str],
    title: str,
    ylabel: str,
    filename: str,
) -> None:
    seeds = [row["seed"] for row in rows]

    fig, ax = plt.subplots(figsize=(8, 4.5))
    palette = {
        "early": "#ff9800",
        "mid": "#3f51b5",
        "late": "#009688",
    }

    for phase, column in column_map.items():
        series = [row[column] for row in rows]
        ax.plot(seeds, series, marker="o", linestyle="-", label=phase.title(), color=palette[phase])

    ax.set_title(title)
    ax.set_xlabel("Seed")
    ax.set_ylabel(ylabel)
    ax.grid(True, linestyle=":", alpha=0.4)
    ax.legend(loc="best")
    fig.tight_layout()
    fig.savefig(out_dir / filename, dpi=160)
    plt.close(fig)


def _plot_upgrade_timing(rows: List[Dict[str, float]], out_dir: Path) -> None:
    seeds = [row["seed"] for row in rows]
    upgrades = [row.get("first_upgrade_hour", float("nan")) for row in rows]

    fig, ax = plt.subplots(figsize=(8, 4.5))
    ax.scatter(seeds, upgrades, color="#795548")
    ax.set_title("First Upgrade Timing")
    ax.set_xlabel("Seed")
    ax.set_ylabel("Hour of First Fleet Upgrade")
    ax.grid(True, linestyle=":", alpha=0.4)
    fig.tight_layout()
    fig.savefig(out_dir / "sanity_first_upgrade.png", dpi=160)
    plt.close(fig)


def _write_summary(rows: List[Dict[str, float]], out_dir: Path) -> None:
    summary = {
        "feasible_ratio": _summarise(row["feasible_ratio"] for row in rows),
        "upgrade_hour": _summarise(row["first_upgrade_hour"] for row in rows),
        "avg_route_len": _summarise(row["avg_route_len"] for row in rows),
        "phase_margin_per_hour": {
            phase: _summarise(row[column] for row in rows)
            for phase, column in {
                "early": "phase_early_margin_per_hour",
                "mid": "phase_mid_margin_per_hour",
                "late": "phase_late_margin_per_hour",
            }.items()
        },
        "phase_cash_gain": {
            phase: _summarise(row[column] for row in rows)
            for phase, column in {
                "early": "phase_early_cash_gain",
                "mid": "phase_mid_cash_gain",
                "late": "phase_late_cash_gain",
            }.items()
        },
    }

    with (out_dir / "sanity_summary.json").open("w", encoding="utf-8") as handle:
        json.dump(summary, handle, indent=2)


def process_scenario(scenario: str, output_root: Path) -> None:
    scenario_dir = output_root / scenario
    scenario_dir.mkdir(parents=True, exist_ok=True)
    rows = _read_seed_metrics(scenario, output_root)

    _plot_feasibility(rows, scenario_dir)
    _plot_phase_metric(
        rows,
        scenario_dir,
        {
            "early": "phase_early_margin_per_hour",
            "mid": "phase_mid_margin_per_hour",
            "late": "phase_late_margin_per_hour",
        },
        "Margin per Flight Hour",
        "Net Margin ($/h)",
        "sanity_phase_margins.png",
    )
    _plot_phase_metric(
        rows,
        scenario_dir,
        {
            "early": "phase_early_cash_gain",
            "mid": "phase_mid_cash_gain",
            "late": "phase_late_cash_gain",
        },
        "Cash Gain by Phase",
        "Cash Gained ($)",
        "sanity_phase_cash.png",
    )
    _plot_upgrade_timing(rows, scenario_dir)
    _write_summary(rows, scenario_dir)


def main() -> None:
    parser = argparse.ArgumentParser(description="Render sanity benchmark graphs")
    parser.add_argument(
        "--scenario-config",
        type=Path,
        default=DEFAULT_CONFIG,
        help="Scenario config to evaluate (defaults to benchmarks/sanity.yaml)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=DEFAULT_OUTPUT,
        help="Benchmark output directory (defaults to benchmarks/outputs)",
    )
    parser.add_argument(
        "--skip-run",
        action="store_true",
        help="Skip invoking run_benchmarks.py and only render existing results",
    )
    args = parser.parse_args()

    config_path = args.scenario_config.resolve()
    output_root = args.output_dir.resolve()

    if not args.skip_run:
        _maybe_run_benchmarks(config_path)

    scenarios = _load_config(config_path)
    for scenario in scenarios:
        process_scenario(scenario, output_root)


if __name__ == "__main__":
    main()
