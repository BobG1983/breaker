# ChainLightning

Arc damage jumping sequentially between nearby cells with visual lightning arcs.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `arcs` | `u32` | Number of jumps (each jump damages one cell) |
| `range` | `f32` | Maximum jump distance from current source to next target |
| `damage_mult` | `f32` | Damage multiplier per arc (1.x format) |
| `arc_speed` | `f32` | Arc travel speed in world units per second (default: 200.0) |

## Behavior

When triggered on a target:

1. **Damage initial target**: the triggered cell takes damage immediately. Damage per hit = base bolt damage × `damage_mult`.
2. **Target becomes source**: the damaged cell is now the arc origin for the next jump.
3. **Pick next target**: use RNG to select a random cell within `range` of the current source, excluding cells already hit in this chain.
4. **Draw lightning arc**: spawn a visual arc entity from source to new target. The arc animates/travels from source to target.
5. **Arc arrives**: when the arc reaches the target, that target takes damage and becomes the new source.
6. **Decrement jumps**: reduce remaining jump count by 1.
7. **Repeat** steps 3–6 until jumps are exhausted or no valid targets remain within range.

Each cell is hit at most once per chain. The chain plays out over multiple frames — it is not instant. If no valid targets exist in range at any point, the chain ends early.

## Reversal

No-op. Chain and arc entities self-despawn on their own lifecycle.
