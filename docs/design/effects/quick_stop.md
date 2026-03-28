# QuickStop

Breaker deceleration multiplier — enables precise stops at high speed.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `multiplier` | `f32` | Deceleration multiplier (1.x format: 2.0 = 2x faster deceleration) |

## Behavior

Pushes the multiplier to the entity's `ActiveQuickStops` vec. When the breaker is dashing and the player inputs the opposite direction, the breaker "quick stops" — the deceleration phase is accelerated by the combined multiplier, and the tilt animation plays faster (effectively skipping some of the dash and speeding the settle/brake phase).

The breaker movement system reads `ActiveQuickStops` to determine deceleration speed when a direction reversal is detected.

## Stacking

Multiplicative. Multiple boosts multiply together.

## Reversal

Removes the matching multiplier entry from `ActiveQuickStops`.

## Evolution: FlashStep

On a successful quick stop, the breaker is immediately teleported to the X position directly under the lowest active bolt's Y position. Enables "teleport-and-plant" playstyle at high stacks.
