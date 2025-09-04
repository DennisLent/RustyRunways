---
title: Airplane Models
---

# Airplane Models

RustyRunways features a range of aircraft tuned for different roles and budgets. All models are defined in `utils::airplanes::models` and expose specs via `AirplaneModel::specs()`.

## Available Models and Specs

Each row lists the model’s hard specs from code and the computed minimum runway requirement (meters) derived from simplified physics in the core crate.

| Model           | MTOW (kg) | Cruise (km/h) | Fuel (L) | Burn (L/h) | Oper. Cost ($/h) | Payload (kg) | Price ($)   | Min Runway (m) |
|-----------------|-----------:|---------------:|---------:|-----------:|-----------------:|-------------:|------------:|---------------:|
| SparrowLight    |      5000 |            250 |      200 |         30 |              300 |          500 |     200,000 |            408 |
| FalconJet       |      8000 |            800 |     2000 |        250 |            1,500 |        1,500 |   1,500,000 |          4,173 |
| CometRegional   |     20000 |            700 |     5000 |        600 |            3,000 |        5,000 |  10,000,000 |          3,195 |
| Atlas           |     40000 |            750 |    12000 |      1,500 |            6,000 |       15,000 |  30,000,000 |          3,668 |
| TitanHeavy      |    100000 |            650 |    20000 |      3,000 |           10,000 |       50,000 |  60,000,000 |          2,753 |
| Goliath         |    200000 |            550 |    40000 |      6,000 |           20,000 |      100,000 | 120,000,000 |          1,972 |
| Zephyr          |     50000 |            900 |    25000 |      1,200 |            8,000 |       25,000 |  50,000,000 |          5,281 |
| Lightning       |     15000 |           1800 |     5000 |      1,000 |           12,000 |        2,000 |  80,000,000 |         21,125 |

Note:

- Minimum runway is computed as the max of takeoff and landing distances using assumptions from the code. With the current parameters, takeoff distance dominates: `min_runway ≈ (0.65 · cruise_mps)² / (2·2.5)` and `cruise_mps = cruise_kmh / 3.6`.
- Values are rounded to the nearest meter for readability.

## Fields Reference

- `mtow`: maximum take‑off weight (kg)
- `cruise_speed`: km/h
- `fuel_capacity`: liters
- `fuel_consumption`: liters per hour
- `operating_cost`: $/hour
- `payload_capacity`: kg
- `purchase_price`: $
- `min_runway_length`: meters, computed from cruise speed with fixed acceleration/deceleration constants
