# ChainBolt

Spawns a chain bolt tethered to the entity via a distance constraint.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `tether_distance` | `f32` | Maximum distance from anchor entity |

## Behavior

Spawns a new bolt entity at the anchor's position with a randomized velocity. Creates a DistanceConstraint linking the new bolt to the anchor entity, preventing it from traveling further than `tether_distance`.

## Reversal

Despawns the chain bolt and its distance constraint if still alive.
