# SpeedBoost

Scales the entity's speed by a multiplier.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `multiplier` | `f32` | Speed multiplier (1.x format: 1.5 = 50% faster, 2.0 = 2x) |

## Behavior

Pushes the multiplier to the entity's ActiveSpeedBoosts vec. A recalculation system computes `base_speed * product(all_boosts)`, clamped to `[min, max]`.

## Stacking

Multiplicative. Multiple boosts multiply together.

## Reversal

Removes the matching multiplier entry from ActiveSpeedBoosts.
