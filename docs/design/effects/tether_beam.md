# TetherBeam

Two free-moving bolts connected by a crackling neon beam that damages everything it intersects.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `damage_mult` | `f32` | Damage multiplier for beam contact (1.x format) |

## Behavior

Evolution of ChainBolt. Spawns two bolts that move freely (no distance constraint). A crackling neon/electric beam connects them visually. The beam is a line segment between the two bolt positions — any cell whose bounds intersect the beam segment takes damage each tick. Each cell is damaged at most once per tick. Players position both bolts to maximize diagonal beam sweep across cell clusters.

## Reversal

No-op. The beam bolts have their own lifecycle (despawned when lost like any bolt).
