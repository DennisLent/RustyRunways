# Benchmark Harness

The benchmarking harness drives the Python bindings to simulate many seeds across configurable scenarios. It lives entirely in this directory so that the data, plots, and configuration live alongside the driver script.

## Getting Started

Run `./setup.sh` once to create a virtual environment with the Python bindings and plotting dependencies compiled. Activate the environment with `source .venv/bin/activate`, then execute `python run_benchmarks.py` with any command line options you need. The script prints a progress bar per scenario, followed by per-seed statistics and aggregate summaries.

## Supplying Scenarios

Large experiments are described in a YAML file passed with `--scenario-config`. The file contains optional `defaults` and a `scenarios` list. Defaults provide world knobs that every scenario inherits unless it overrides them. Typical fields include `hours`, `seeds`, `cash`, `num_airports`, and any nested `gameplay` values such as restock cadence, fuel interval, or specific `orders` parameters (e.g., `min_weight`, `max_weight`, `alpha`, `beta`).

Each item in `scenarios` can:

- reference an existing world on disk with `config_path`, or
- specify a `world` override (through plain YAML nesting) that the harness will materialise into a temporary configuration which is deleted after the run.

Scenarios may vary the parameters in several ways:

- `sweep`: define a single dotted path (such as `gameplay.orders.min_weight`) and a list of values. The harness expands the sweep into separate variants, runs them, and records the varied parameter in the summary output. Multiple sweeps can be attached to a scenario by supplying a list. The example configuration demonstrates restock, starting cash, airport count, min-weight, max-weight, `alpha`, and `beta` sweeps.
- `variants`: provide named overrides for bespoke combinations. This is useful when pairs of values must change together (for instance, `min_weight` and `max_weight`) to stay within plausible ranges. Each variant may add optional `metadata` to describe the combination.
- `include_base`: set to `false` when you only want the explicit variants and not the inherited default world.

Seeds and horizon hours can be assigned globally, per scenario, or per variant. When none are given, the command line defaults supplied to `run_benchmarks.py` take effect.

## Outputs

All artifacts are written under `benchmarks/outputs`. Each scenario (and each expanded variant) receives its own subdirectory containing:

- `seed_<n>_timeline.csv` files with hour-by-hour cash, fleet size, and deliveries.
- PNG plots that show cash, fleet size, and deliveries over time for every seed.
- A `margin_per_hour.png` bar chart that compares the profitability of each seed.

In addition to the per-scenario folders, the root of `outputs` contains:

- `scenario_summary.csv`, which aggregates the results per variant (averaged across seeds) and records the parameter value that was swept.
- A `summary/` directory holding comparison charts generated from sweeps. When a parameter is numeric, the harness produces a line plot showing each metric against the parameter value. For categorical or labelled variants, it renders bar charts.

These files are intended to be disposable. You can safely delete the entire `outputs` directory between runs if you want a clean slate.

## Example Configuration

See `scenarios.example.yaml` in this directory for a ready-to-run template. It includes the default gameplay settings drawn from the sample world and demonstrates sweeps over restock cadence, starting cash, airport counts, minimum and maximum order weights, and the `alpha`/`beta` parameters. It also provides explicit weight combinations so you can observe how paired adjustments behave compared to isolated changes.

## Greedy Agent

The bundled policy in `run_benchmarks.py` operates with a few simple heuristics. It inspects parked airplanes one at a time, unloading any cargo that just arrived, refuelling when a prospective trip would consume more than ninety percent of the remaining range, and ranking visible orders by value per kilometre subject to payload and range checks. Once an order is selected it spends an hour loading, records the route distance, and departs toward the destination. When every aircraft is tied up (loading, refuelling, in transit, or undergoing maintenance) the harness advances time in one-hour increments until another decision is needed; mechanical failures trigger maintenance immediately.

The agent keeps an eye on the player's cash and, when the runway allows and funds comfortably exceed the purchase price, buys a second SparrowLight to increase throughput. Additional aircraft are left for future experimentation—the goal here is a deterministic baseline that finishes quickly so the sweeps remain approachable.
