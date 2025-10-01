---
title: Airplane Models
---

# Airplane Models

RustyRunways features a range of aircraft tuned for different roles and budgets. All models are defined in `utils::airplanes::models` and expose specs via `AirplaneModel::specs()`.

## Available Models and Specs

Each row lists the model’s specs as defined in the code and the computed minimum runway requirement (meters) derived from simplified physics in the core crate. The role and passenger capacity columns reflect the new passenger/cargo support: cargo-only aircraft have `0` seats, passenger-only aircraft have non‑zero seats and a smaller or zero payload capacity, and mixed “combi” aircraft support both.

| Model              | Role       | MTOW (kg) | Cruise (km/h) | Fuel (L) | Burn (L/h) | Oper. Cost ($/h) | Payload (kg) | Pax (seats) | Price ($)   | Min Runway (m) |
|--------------------|------------|-----------:|---------------:|---------:|-----------:|-----------------:|-------------:|------------:|------------:|---------------:|
| SparrowLight       | Mixed      |      5,200 |            260 |      240 |         35 |              340 |        1,200 |           6 |     240,000 |            441 |
| FalconJet          | Passenger  |      8,300 |            780 |    2,200 |        260 |            1,600 |          600 |          12 |   1,700,000 |          3,967 |
| CometRegional      | Passenger  |     24,000 |            720 |    6,000 |        620 |            3,200 |        4,000 |          78 |  12,000,000 |          3,380 |
| Atlas              | Mixed      |     42,000 |            750 |   12,500 |      1,550 |            6,500 |       18,000 |          68 |  34,000,000 |          3,668 |
| TitanHeavy         | Cargo      |    110,000 |            670 |   22,000 |      3,200 |           11,000 |       55,000 |           0 |  68,000,000 |          2,927 |
| Goliath            | Cargo      |    210,000 |            580 |   45,000 |      6,500 |           22,000 |      110,000 |           0 | 130,000,000 |          2,193 |
| Zephyr             | Passenger  |     82,000 |            900 |   28,000 |      1,450 |            9,000 |        8,000 |         210 |  72,000,000 |          5,281 |
| Lightning          | Passenger  |     18,500 |          1,800 |    5,400 |      1,100 |           12,000 |        1,500 |          32 |  88,000,000 |         21,125 |
| BisonFreighter     | Cargo      |     28,000 |            680 |    8,500 |        900 |            4,800 |       20,000 |           0 |  18,000,000 |          3,015 |
| TrailblazerCombi   | Mixed      |     65,000 |            820 |   18,000 |      1,800 |            7,500 |       25,000 |         120 |  55,000,000 |          4,384 |

Note:

- Minimum runway is computed as the max of takeoff and landing distances using assumptions from the code. With the current parameters, takeoff distance dominates: `min_runway ≈ (0.65 · cruise_mps)² / (2·2.5)` and `cruise_mps = cruise_kmh / 3.6`.
- Values are rounded to the nearest meter for readability.

## Fields Reference

- `mtow`: maximum take‑off weight (kg)
- `cruise_speed`: km/h
- `fuel_capacity`: liters
- `fuel_consumption`: liters per hour
- `operating_cost`: $/hour
- `payload_capacity`: kg (cargo)
- `passenger_capacity`: seats (people)
- `role`: Cargo / Passenger / Mixed
- `purchase_price`: $
- `min_runway_length`: meters, computed from cruise speed with fixed acceleration/deceleration constants
