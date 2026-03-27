# DamageBoost

Multiplicative damage bonus.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `boost` | `f32` | Damage multiplier (1.x format: 2.0 = 2x damage) |

## Behavior

Pushes the multiplier to the entity's ActiveDamageBoosts vec. Damage calculation uses `base_damage * product(all_boosts)`.

## Stacking

Multiplicative. Multiple boosts multiply together.

## Reversal

Removes the matching multiplier entry from ActiveDamageBoosts.
