# 5i: Cell Visuals

**Goal**: Give each cell type a distinct shape and color identity, implement damage state visualization, and add destruction effects.

## What to Build

### 1. Per-Type Cell Shapes

Replace flat rectangles with per-type meshes:

| Cell Type | Shape | Visual Read |
|-----------|-------|-------------|
| Standard | Rectangle/RoundedRectangle | Clean, simple — the baseline |
| Tough | Hexagon | Heavier, more substantial, brighter/denser glow |
| Lock | Octagon | Amber/gold with visible lock glyph, glyph dissolves on unlock |
| Regen | Circle | Green-tinted, pulsing animation (cell visibly breathes) |

Shapes driven by the entity's `Shape` component (from 5f rendering/ enums, set at spawn from RON).

### 2. Cell Color System

Colors driven by the entity's `Color` component:
- `TemperatureDefault`: Follows run temperature palette (from 5f)
- Fixed colors (CoolBlue, WarmAmber, Gold, etc.): Override temperature
- All cells glow with HDR emissive for bloom interaction (from 5d)

### 3. Damage State Visualization

Each cell communicates health through its `DamageDisplay` enum:

| Display Mode | Visual |
|--------------|--------|
| Fracture | Cracks appear and grow with damage |
| Fade | Brightness/opacity decreases with damage |
| Flicker | Cell flickers more frequently as health drops |
| Shrink | Cell physically shrinks with damage |
| ColorShift | Color shifts toward red/dim as health drops |

### 4. CellRenderState Component

Defined in cells/ domain, synced when health changes:
- `health_fraction: f32` — 0.0-1.0
- `is_locked: bool` — for lock cell visual state
- `is_regenerating: bool` — for regen pulse timing

### 5. Cell Destruction Effects

Context-adaptive death effects (currently cells despawn instantly):

| Context | DeathEffect | Visual |
|---------|------------|--------|
| Single cell break | Dissolve | Clean fade with Spark burst |
| Combo chain (2-4 rapid kills) | Shatter | Fracture into Shard particles |
| Chain reaction (5+ kills) | EnergyRelease | Expanding Energy Ring + bright flash |

rendering/ determines context from destruction event frequency/proximity.

### 6. Cell Hit Impact

When bolt hits a cell:
- 4-8 Spark particles from impact point (angle matches bolt direction)
- Cell micro-flash (HDR ~1.3, 1-2 frames)
- Fracture-display cells gain a crack at impact point
- Scales with damage dealt

### 7. Lock Cell Unlock VFX

When a lock cell is unlocked:
- Lock glyph fractures outward (~0.2s)
- Golden Shard particles scatter
- Cell transitions amber → true color
- Energy Ring + flash (HDR ~1.5)

### 8. Regen Cell Healing Pulse

When a regen cell heals:
- Green glow pulse outward on each heal tick (~1.3x cell radius)
- Glow Mote particles drift upward
- Fracture cracks visibly seal with green glow

### 9. Shield Cell (Orbiting) Visual

Shield cells orbiting a parent:
- Smaller, brighter than parent
- Visible orbit ring trail (faint Trail particles)
- Brightness distinct from parent cell

### 10. Powder Keg Modifier Visual

When a cell has the Powder Keg chip modifier:
- Flickering, sparking visual overlay
- Looks volatile/unstable
- Spark particles emit intermittently

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing/bloom), 5e (particles: Spark, Shard, Energy Ring, Trail, Glow Mote), 5f (visual composition enums)
- **Independent of**: 5g, 5h, 5j (other entity visuals)

## What This Step Builds

- Per-type cell shapes (Rectangle, Hexagon, Octagon, Circle) with custom meshes
- Cell color system (temperature-following + fixed overrides)
- 5 damage display modes (Fracture, Fade, Flicker, Shrink, ColorShift)
- CellRenderState component (synced on health change)
- Context-adaptive destruction VFX (Dissolve/Shatter/EnergyRelease based on combo context)
- Cell hit impact particles (directional sparks + micro-flash)
- Lock cell unlock VFX (glyph fracture + golden shards + color transition)
- Regen cell healing pulse (green glow + motes + crack sealing)
- Shield cell orbit visuals (distinct brightness + orbit trail)
- Powder Keg modifier overlay (flickering/sparking)

## Verification

- Each cell type has a distinct shape
- Damage progression is visually clear per DamageDisplay enum
- Destruction effects differ by context (single, combo, chain)
- Cell hit produces sparks in correct direction
- Lock unlock and regen pulse VFX fire correctly
- All existing tests pass
