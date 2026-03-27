# Pulse

Fires a shockwave at every active bolt position simultaneously.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `base_range` | `f32` | Base radius per shockwave |
| `range_per_level` | `f32` | Extra radius per stack |
| `stacks` | `u32` | Stack count |
| `speed` | `f32` | Expansion speed in world units/sec |

## Behavior

Queries all bolt entities and spawns a shockwave at each bolt's position. Functionally equivalent to a Shockwave on every bolt at once.

## Reversal

No-op. Pulse shockwave entities self-despawn on their own lifecycle.
