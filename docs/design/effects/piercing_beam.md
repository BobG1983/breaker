# PiercingBeam

A fast-expanding beam rectangle in the entity's velocity direction.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `damage_mult` | `f32` | Damage multiplier (1.x format) |
| `width` | `f32` | Beam width in world units |

## Behavior

Fires all damage in a single tick via a deferred `PiercingBeamRequest` entity. All cells along the bolt's velocity direction within `width` are damaged simultaneously. Each cell is damaged at most once per beam. Damage per cell = base bolt damage * `damage_mult`. The request entity is consumed after processing.

## Reversal

No-op. Beam entities self-despawn on their own lifecycle.
