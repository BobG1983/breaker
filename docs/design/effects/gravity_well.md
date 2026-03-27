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

Spawns a gravity well entity at the entity's position. Bolts within `radius` are pulled toward the well center. Despawns after `duration` seconds. If `max` wells already exist, the oldest is despawned first.

## Reversal

Despawns the gravity well entity if still alive.
