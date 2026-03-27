# Explode

Instant area damage centered on the entity's position. Unlike [Shockwave](shockwave.md), damage is applied immediately to all cells in range — there is no expanding wavefront. Uses a distinct visual effect (flash/burst rather than expanding ring).

**Status**: Not yet implemented.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `range` | `f32` | Blast radius in world units |
| `damage_mult` | `f32` | Damage multiplier applied to each cell in range |

## Behavior

Instantly damages all cells within `range` of the entity's position. Damage per cell = base bolt damage * `damage_mult`. 

## Visual

Flash/burst graphic at the entity position — distinct from Shockwave's expanding ring. Should feel like a detonation, not a ripple.

## Reversal

No reversal — damage is instant and cannot be undone.
