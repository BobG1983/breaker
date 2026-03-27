# ChainLightning

Arc damage jumping between nearby cells using greedy nearest-neighbor traversal.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `arcs` | `u32` | Number of jumps |
| `range` | `f32` | Maximum jump distance between cells |
| `damage_mult` | `f32` | Damage multiplier per arc (1.x format) |

## Behavior

Starting from the entity's position, finds the nearest cell within range and damages it. Then jumps to the nearest undamaged cell within range of that cell, repeating for `arcs` jumps. Each cell is hit at most once per chain.

## Reversal

No-op. Lightning arc entities self-despawn on their own lifecycle.
