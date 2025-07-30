# Commands

The game can be interacted with using a Domain-specific language (DSL). This make it easier, as queries or commands can be broken down into a more manageable tokens. 

## Inspecting the world state

`SHOW AIRPORTS`

`SHOW AIRPORTS WITH ORDERS`

`SHOW AIRPORTS <airport_id>`: show full details & orders

`SHOW AIRPORTS <airport_id> WITH ORDERS`: only orders at that airport

`SHOW PLANE`: show players entire fleet

`SHOW PLANE <plane_id>`: show one plane (status, specs, manifest)

`SHOW DISTANCES <plane_id>`: shows the distances, fuel requirements and if it can land at  given airport

## Purchases

`BUY PLANE <Model> <airport_id>`: Buys and places an airplane at the given airport

## Cargo handling

`LOAD ORDER <order_id> ON <plane_id>`: Load 1 order onto the plane (takes 1 hour)

`LOAD ORDERS [<order_id>] ON <plane_id>`: Loads n orders onto the plane (takes 1 hour)

`UNLOAD ORDER <order_id> FROM <plane_id>`: Load 1 order from the plane (takes 1 hour)

`UNLOAD ORDERs [<order_id>] FROM <plane_id>`: Load 1 order from the plane (takes 1 hour)

`UNLOAD ALL FROM <plane_id>`: Unload all orders from the plane (takes 1 hour)

## Dispatch & movement

`DEPART PLANE <plane_id> <destination_airport_id>`: Sends a specific airplane on its way to the destination airport

`HOLD PLANE <plane_id>`: Plane stays parked at current location

## Time control

`ADVANCE <n>`: Advances the game by n hours (ticks) or until a new event occurs

The game can also be manually progressed by 1 hour by pressing the Enter / Return Key (i.e. no input)

## Queries

`SHOW CASH`: Shows the cash reserves of the player

`SHOW TIME`: Shows the current GameTime

## Exit

`EXIT`