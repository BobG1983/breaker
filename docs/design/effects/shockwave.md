# Shockwave

Expanding ring of area damage centered on the entity's position.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `base_range` | `f32` | Base radius before stacking |
| `range_per_level` | `f32` | Extra radius per stack beyond the first |
| `stacks` | `u32` | Current stack count (starts at 1) |
| `speed` | `f32` | Expansion speed in world units/sec |

Effective range = `base_range + (stacks - 1) * range_per_level`.

## Behavior

Spawns a wavefront entity at the entity's position that expands outward. Damages cells within the expanding ring. Despawns when fully expanded.

## Reversal

Despawns the shockwave entity if still alive.
