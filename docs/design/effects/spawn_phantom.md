# SpawnPhantom

Spawns a temporary phantom bolt with infinite piercing and a lifespan timer.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `duration` | `f32` | Lifespan in seconds |
| `max_active` | `u32` | Maximum phantom bolts alive at once |

## Behavior

Spawns a phantom bolt entity that passes through all cells (infinite piercing) and despawns after `duration` seconds. If `max_active` phantoms already exist, the oldest is despawned first.

## Reversal

Despawns the phantom bolt entity if still alive.
