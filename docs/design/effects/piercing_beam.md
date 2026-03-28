# PiercingBeam

A fast-expanding beam rectangle in the entity's velocity direction.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `damage_mult` | `f32` | Damage multiplier (1.x format) |
| `width` | `f32` | Beam width in world units |

## Behavior

Spawns a thin rectangle entity at the entity's position that expands very quickly along the entity's velocity direction. Damages all cells the rectangle touches as it expands. Each cell is damaged at most once per beam. Damage per cell = base bolt damage * `damage_mult`. The beam entity despawns after reaching the playfield boundary.

## Reversal

No-op. Beam entities self-despawn on their own lifecycle.
