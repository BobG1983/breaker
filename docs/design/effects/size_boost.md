# SizeBoost

Increases the entity's size — varies by entity type.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `value` | `f32` | Size multiplier (1.x format) |

## Behavior

Pushes the multiplier to the entity's `ActiveSizeBoosts` vec. A recalculation system computes the effective size. Behavior varies by entity type:

- **Breaker**: increases width only
- **Cell**: increases full scale (width and height)
- **Bolt**: increases radius
- **Wall**: no-op

## Stacking

Multiplicative. Multiple boosts multiply together.

## Reversal

Removes the matching multiplier entry from `ActiveSizeBoosts`.
