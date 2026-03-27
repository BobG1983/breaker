# Shield

Temporary breaker protection.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `base_duration` | `f32` | Base duration in seconds |
| `duration_per_level` | `f32` | Extra duration per stack |
| `stacks` | `u32` | Stack count |

Effective duration = `base_duration + (stacks - 1) * duration_per_level`.

## Behavior

Spawns a shield entity on the breaker that blocks bolt loss for the duration. Despawns when the timer expires.

## Reversal

Despawns the shield entity if still alive.
