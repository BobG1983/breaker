# SecondWind

Spawns an invisible bottom wall that bounces the bolt once, preventing bolt loss.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `invuln_secs` | `f32` | Brief invulnerability window after bounce |

## Behavior

Spawns an invisible wall entity at the bottom of the playfield. When the bolt hits it, the bolt bounces back up and the wall is despawned (single use).

## Reversal

Despawns the invisible wall entity if still alive.
