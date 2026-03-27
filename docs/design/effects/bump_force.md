# BumpForce

Flat bump force increase.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `force` | `f32` | Force increase per stack |

## Behavior

Pushes the value to the entity's ActiveBumpForces vec. A recalculation system computes `base_force + sum(all_boosts)`.

## Stacking

Additive. Multiple applications add together.

## Reversal

Removes the matching value entry from ActiveBumpForces.
