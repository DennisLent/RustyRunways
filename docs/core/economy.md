---
title: Economy and Costs
---

# Economy and Costs

RustyRunways tracks income and expenses daily. Player cash changes as you buy planes, pay fees, and deliver orders.

## Cash Flows

- Income: order deliveries credited upon successful unload at destination.
- Expenses: purchase prices, operating costs during flight, landing fees, fuel purchases, parking fees, maintenance.
- The engine maintains `daily_income` and `daily_expenses` aggregates for quick stats.

## Fees and Prices

- Landing fee: `airport.landing_fee(airplane) = airport.landing_fee_base * (MTOW / 1000)`.
- Parking fee: per hour, based on airport size (runway length proxy).
- Fuel price: each airport has `fuel_price` ($/L), generated within `[0.5, 2.5]` and adjusted dynamically.

## Dynamic Fuel Pricing

- Airports track `fuel_sold` and adjust prices every pricing event (e.g., every 6 hours):
  - If fuel was bought: `price *= (1 + 0.05)` (elastic increase)
  - Else: `price += (1 - 0.05)` (drift upwards to encourage activity)
  - Then reset `fuel_sold = 0`.

## Operating Cost and Flights

- Operating cost charged per flight hour using `operating_cost` from the airplane specs.
- Fuel consumption reduces onboard fuel and drives future refueling spend.
- Departures may also include scheduling/administrative overhead expressed via events.

## Strategy Notes

- Choose refueling hubs with cheaper fuel; plan routes to balance runway limits and deadlines.
- Larger airports generate more orders and may pay more but also have higher fees.
- Fleet composition matters: payload capacity, cruise speed, and runway requirement impact profitability.

