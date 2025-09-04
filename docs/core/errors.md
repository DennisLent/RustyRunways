---
title: Errors
---

# Errors

Operations can fail with `GameError`. Messages are designed to be user‑friendly and often include contextual values.

## Error Types

- OutOfRange { distance, range } — requested flight exceeds aircraft range.
- RunwayTooShort { required, available } — destination runway shorter than aircraft requirement.
- MaxPayloadReached { current_capacity, maximum_capacity, added_weight } — loading would exceed payload capacity.
- OrderIdInvalid { id } — no such order at current airport.
- PlaneIdInvalid { id } — no such plane.
- AirportIdInvalid { id } — no such airport.
- AirportLocationInvalid { location } — no airport at coordinate.
- PlaneNotAtAirport { plane_id } — action requires being parked (not in transit).
- PlaneNotReady { plane_state } — current status disallows the action.
- InsufficientFunds { have, need } — not enough cash to complete purchase/operation.
- InsufficientFuel { have, need } — not enough fuel for the requested flight.
- UnknownModel { input, suggestion } — airplane model not recognized; includes suggestion via edit‑distance when close.
- NoCargo — attempted unload but manifest is empty.
- SameAirport — attempted to depart to current airport.
- InvalidCommand { msg } — CLI/Python command parsing failed.

## Recovery Tips

- OutOfRange — refuel en‑route (if possible), fly shorter legs, choose reachable destination, or buy a longer‑range model.
- RunwayTooShort — choose airports with longer runways or different airplane models.
- MaxPayloadReached — unload or choose a heavier‑lift model.
- InsufficientFunds — reduce expenses, deliver more orders, or buy a cheaper plane.
- InsufficientFuel — refuel before departure or at intermediate stops.

