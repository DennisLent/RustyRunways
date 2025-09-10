# Code Coverage

We measure coverage for both the Rust engine and the Python bindings. Our goal is at least 80% line coverage for each.

## Local run

Use the helper script from the repo root:

```
scripts/coverage.sh
```

It runs:

- Rust: `cargo tarpaulin` with a Cobertura XML report in `coverage/rust/cobertura.xml`.
- Python: `pytest --cov` with an XML report in `coverage/python/coverage.xml`.

Thresholds (both default to 80%) can be overridden via env vars:

```
RUST_FAIL_UNDER=85 PY_FAIL_UNDER=85 scripts/coverage.sh
```

## CI

The `Coverage` GitHub Action executes on every PR and on pushes to `main`:

- File: `.github/workflows/coverage.yml`
- Fails the job if either Rust or Python coverage drops below 80%.
- Uploads artifacts under the `coverage/` directory for inspection.

## What’s included

- Rust: All crates except the Python extension crate (`rusty_runways_py`) are measured. Integration tests are executed via Tarpaulin.
- Python: Tests under `crates/py/tests` run against the local dev build (via `maturin develop`).

## Future work

- Frontend (React) coverage: we currently do not include the UI in coverage. We can add Vitest + jsdom and collect coverage from TS/TSX files in `apps/tauri/ui/src` if desired.

