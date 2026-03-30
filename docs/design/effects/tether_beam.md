# TetherBeam

Bolts connected by crackling neon beams that damage everything they intersect.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `damage_mult` | `f32` | Damage multiplier for beam contact (1.x format) |
| `chain` | `bool` | If true, tethers ALL active bolts in sequence instead of spawning 2 new ones |

## Behavior

### Standard mode (`chain: false`)

Evolution of ChainBolt. Spawns two bolts that move freely (no distance constraint). A crackling neon/electric beam connects them visually. The beam is a line segment between the two bolt positions — any cell whose bounds intersect the beam segment takes damage each tick. Each cell is damaged at most once per tick. Players position both bolts to maximize diagonal beam sweep across cell clusters.

### Chain mode (`chain: true`)

Instead of spawning new bolts, tethers ALL existing active bolts in sequence: bolt 1→2→3→4 etc. Creates N-1 beam entities for N bolts. Beams form a chain connecting all bolts on the field.

When a bolt is lost, the chain repairs (bolt 1→3→4 if bolt 2 dies). When a new bolt spawns, it joins the chain at the end.

Bolts are ordered by spawn time for consistent chain topology.

## Reversal

No-op. Beam bolts/entities have their own lifecycle.

## VFX

### Standard mode
- Crackling electric energy along the tether beam
- Elasticity visual — stretches when bolts far apart, slackens when close
- Animated energy flowing along beam (brightness traveling end to end)
- Flash + sparks when tether snaps

### Chain mode (ArcWelder evolution)
- Same crackling energy per beam segment
- Electric corona on all connected bolts
- Chain forms a visible electric web across the playfield when many bolts are active
