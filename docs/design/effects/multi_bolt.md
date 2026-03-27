# MultiBolt

Spawns multiple bolts with stacking.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `base_count` | `u32` | Base bolt count |
| `count_per_level` | `u32` | Extra bolts per stack |
| `stacks` | `u32` | Stack count |

Effective count = `base_count + (stacks - 1) * count_per_level`.

## Behavior

Spawns multiple extra bolts above the breaker. Functionally equivalent to multiple SpawnBolts calls.

## Reversal

Despawns the spawned bolt entities if still alive.
