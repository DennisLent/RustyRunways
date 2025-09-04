---
title: Events and Time
---

# Events and Time

RustyRunways advances in integer hours (`GameTime = u64`). The engine uses a priority queue of scheduled events; the earliest event executes next. Calling `advance(n)` progresses to the target time or until no events remain.

## Event Types

- LoadingEvent { plane }
  - Completes loading an order onto a plane (+1h from when scheduled).
- FlightTakeOff { plane, origin, destination }
  - Marks the start of a flight and transitions plane to `InTransit`.
- FlightProgress { plane }
  - Intermediate progress ticks for long flights (if used by the engine).
- RefuelComplete { plane }
  - Completes refueling and charges fuel costs.
- OrderDeadline { airport, order }
  - Deadline reached for an order; failure penalties may apply.
- Restock
  - Periodic restocking of orders at airports.
- DailyStats
  - Aggregates income/expenses delta for daily reporting.
- DynamicPricing
  - Every 6 hours, adjusts airport fuel prices based on recent demand.
- WorldEvent { airport, factor, duration }
  - Temporary world/airport factor altering prices/fees for `duration` hours.
- WorldEventEnd { airport, factor }
  - Ends a `WorldEvent` and restores base conditions.
- MaintenanceCheck
  - Routine checks that can prevent breakdowns; scheduled regularly.
- Maintenance { plane }
  - Finishes a maintenance action (+1h from scheduling).

## Scheduling Mechanics

- Actions like load/unload/refuel/maintenance schedule their completion at `now + 1h`.
- Departures schedule plane transit and arrival at `now + flight_time`.
- Pricing and restocking are scheduled periodically.

## Advancing Time

- `advance(hours)` repeatedly pops due events and executes them until the target time or queue is empty.
- The game’s `time` is set to the time of the last processed event (or the target if idle).

