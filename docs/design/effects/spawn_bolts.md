# SpawnBolts

Spawns additional bolt entities.

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `count` | `u32` | `1` | Number of bolts to spawn |
| `lifespan` | `Option<f32>` | `None` | Optional lifespan in seconds. `None` = permanent. |
| `inherit` | `bool` | `false` | If true, spawned bolts inherit the parent's BoundEffects |

## Behavior

Spawns `count` extra bolts above the breaker with randomized velocity. Extra bolts despawn on loss (don't respawn). If `inherit` is true, the new bolt gets a copy of the spawning bolt's BoundEffects.

## Reversal

Despawns the spawned bolt entities if still alive.
