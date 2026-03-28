# BumpForce

Multiplicative bump force increase.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `force` | `f32` | Force multiplier (1.x format) |

## Behavior

Pushes the multiplier to the entity's `ActiveBumpForces` vec. A recalculation system computes `base_force * product(all_boosts)`.

## Stacking

Multiplicative. Multiple boosts multiply together.

## Reversal

Removes the matching multiplier entry from `ActiveBumpForces`.
