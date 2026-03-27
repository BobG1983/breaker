# SizeBoost

Increases the entity's size — radius for bolts, width for breakers.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `value` | `f32` | Size increase in world units |

## Behavior

Pushes the value to the entity's ActiveSizeBoosts vec. A recalculation system computes `base_size + sum(all_boosts)`. The handler queries the entity — if it has bolt radius components, adjusts radius. If it has breaker width components, adjusts width.

## Stacking

Additive. Multiple applications add together.

## Reversal

Removes the matching value entry from ActiveSizeBoosts.
