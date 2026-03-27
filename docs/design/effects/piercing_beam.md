# PiercingBeam

Fires a beam through all cells in the entity's current velocity direction.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `damage_mult` | `f32` | Damage multiplier (1.x format) |
| `width` | `f32` | Beam width in world units |

## Behavior

Casts a beam from the entity's position along its velocity direction. Damages all cells intersecting the beam. The beam is instantaneous (not a projectile).

## Reversal

Despawns the beam entity if still alive.
