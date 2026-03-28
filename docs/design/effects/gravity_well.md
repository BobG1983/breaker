# GravityWell

Spawns a gravity well that attracts bolts within radius.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `strength` | `f32` | Pull strength |
| `duration` | `f32` | How long the well lasts in seconds |
| `radius` | `f32` | Attraction radius in world units |
| `max` | `u32` | Maximum active wells at once |

## Behavior

Spawns a gravity well entity at the entity's position. Bolts within `radius` are pulled toward the well center each tick (force applied to velocity). Despawns after `duration` seconds. If `max` wells already exist for this owner, the oldest is despawned first.

## Reversal

No-op. Gravity wells self-despawn via their duration timer.
