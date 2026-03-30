# 5j: Walls & Background

**Goal**: Make the playfield boundaries visible and the background alive, establishing the spatial canvas that all foreground elements exist within.

## What to Build

### 1. Wall Meshes

Current: Invisible collision entities.

Target:
- Thin glowing border line along each playfield edge (left, right, ceiling)
- Very subtle glow at rest (<0.3 HDR, background grid brightness range)
- Color follows temperature palette (from 5f)

### 2. Wall Bolt-Impact Flash

When bolt hits a wall:
- Brief pulse/flash at impact point
- Pulse travels a short distance along the wall, then fades
- Wall "registers" the hit visually

### 3. Bottom Wall Shield Barrier

When `ShieldActive` or `SecondWind` is active:
- Visible energy barrier along the bottom playfield edge
- Brighter than normal wall glow
- Shimmering/rippling animation
- Cracks appear when a charge is consumed
- Last charge break: barrier shatters with Shard particles
- **Patterned white** — pulsing white with hexagonal/honeycomb pattern, distinguished by pattern not color

### 4. Background Grid

Current: Pure void (ClearColor only).

Target:
- Flat 2D grid — straight horizontal and vertical lines forming a regular grid
- Very dim — barely visible against the void
- Grid line density **configurable via debug menu** — start with medium density, tune in-engine once distortion effects exist. Stored in `RenderingDefaults` RON.
- Temperature-following tint (from 5f)

The grid is a **passive reference surface** — it does NOT warp or react to game events. Screen-space shader effects (from 5d) warp the rendered screen, which incidentally warps the grid as viewed.

### 5. Background Energy Sprites

Occasional bright points traveling along grid lines:
- Glow Mote particles (from 5e) that follow grid paths
- Very subtle — 1-2 visible at any time
- Slow constant speed
- Sense that the grid is alive without being distracting

### 6. Void Background Color

Current: `ClearColor` dark blue-purple — slightly too purple.

Target: Near-black (#050510 range) — deep blue-black that allows gravity well voids to register as even darker.

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing for bloom), 5e (particles: Glow Mote, Shard), 5f (temperature palette)
- DR-3 resolved: patterned white shield. DR-6 resolved: configurable grid density.
- **Independent of**: 5g, 5h, 5i (other entity visuals)

## What This Step Builds

- Wall meshes (thin glowing border lines, temperature-tinted)
- Wall bolt-impact flash (pulse at impact point, travels along wall)
- Shield barrier (patterned white hexagonal, shimmering, cracks on charge loss, shatter on last)
- Background grid (configurable density, temperature-tinted, very dim)
- Background energy sprites (Glow Mote particles along grid lines)
- Corrected void background color (#050510 deep blue-black)

## Verification

- Walls are visible as subtle glowing borders
- Wall flash triggers on bolt impact
- Shield barrier visible when ShieldActive is present
- Background grid renders at correct density and temperature tint
- Energy sprites travel along grid lines
- Void color is correct deep blue-black
- All existing tests pass
