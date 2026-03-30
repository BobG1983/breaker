# 5h: Breaker Visuals

**Goal**: Give each breaker archetype a fully distinct visual identity — different shape, color, aura, and dash trail.

## What to Build

### 1. Per-Archetype Shapes

Replace the flat `Rectangle::new(1.0, 1.0)` with archetype-specific meshes/shaders:

| Archetype | Shape | Description |
|-----------|-------|-------------|
| Aegis | Shield | Wider, protective, convex front face |
| Chrono | Angular | Sleek, sharp geometric edges |
| Prism | Crystalline | Faceted, multi-angled, refractive |

Each shape uses a custom Material2d with additive blending and HDR glow.

### 2. Color Accent System

Per-archetype signature color applied to aura, trail, bump flash:
- Aegis: Blue/cyan
- Chrono: Amber/gold
- Prism: Magenta/violet

Colors read from the entity's `Color` component (set from RON data in 5f).

### 3. Aura System (Idle)

Ambient effect around the breaker at rest, per-archetype:

| Archetype | Aura | Description |
|-----------|------|-------------|
| Aegis | ShieldShimmer | Defensive energy field visible around breaker |
| Chrono | TimeDistortion | Rippling time-echo effect, subtle afterimage at rest |
| Prism | PrismaticSplit | Rainbow edge refractions, light scatters from surfaces |

Aura renders as a shader effect on the breaker's Material2d, not as separate entities.

### 4. Dash Trail System

Visual left behind during dash, per-archetype:

| Archetype | Trail | Description |
|-----------|-------|-------------|
| Aegis | ShieldEnergy | Solid, protective-feeling wake |
| Chrono | Afterimage | Fading copies showing recent positions |
| Prism | PrismaticSplit | Trail separates into spectral colors |

Uses Trail particle type from 5e for base implementation, with per-archetype color/behavior.

### 5. Breaker States

Visual transitions between states:

| State | Visual |
|-------|--------|
| Idle | Base appearance with ambient aura |
| Moving | Aura intensifies in movement direction. Tilt rotation (currently in codebase) replaced by rendering/ implementation. |
| Dashing | Full trail active, glow intensifies, archetype-specific dash VFX |
| Settling (post-dash) | Trail fading, aura returning to idle intensity |

### 6. BreakerRenderState Component

Defined in breaker/ domain, synced each frame:
- `state: BreakerMovementState` — idle, moving, dashing, settling
- `velocity: f32` — current speed for visual intensity
- `tilt: f32` — current tilt angle
- `dash_progress: f32` — 0.0-1.0 during dash for trail timing

### 7. Bump Pop

Replace current placeholder Y-offset pop with full VFX:
- Scale overshoot at peak (punch scale)
- Brief archetype-color flash at contact point

### 8. Width Boost Visual

Replace current placeholder scale-only behavior with full VFX:
- Brief stretch animation on width change
- Aura pulse on activation

### 9. Speed Boost Visual

When breaker has speed boost chip effect:
- Aura stretches in movement direction, trailing wisps
- Dash trail activates at lower intensity during normal movement
- Speed lines at high stacks

### 10. Bump Force Visual

When breaker has bump force boost:
- Front face gains intensified archetype-color glow, pulsing slowly
- White-hot at high stacks (HDR >1.0)
- Flares sharply on bump contact

## Dependencies

- **Requires**: 5c (rendering/ domain), 5d (post-processing), 5e (particles for trails), 5f (visual composition enums)
- **Independent of**: 5g, 5i, 5j (other entity visuals)

## What This Step Builds

- 3 distinct archetype shapes (Shield/Angular/Crystalline) with custom Material2d
- Per-archetype color accents (BlueCyan/Amber/Magenta)
- Per-archetype aura system (ShieldShimmer/TimeDistortion/PrismaticSplit)
- Per-archetype dash trail system (ShieldEnergy/Afterimage/PrismaticSplit)
- Breaker state transitions (idle → moving → dashing → settling)
- BreakerRenderState component (synced each frame by breaker/ domain)
- Bump pop VFX (scale overshoot + archetype flash)
- Width/speed/force boost visuals

## Verification

- All three archetypes have distinct shapes, colors, auras, and dash trails
- Breaker states (idle, moving, dashing, settling) produce visible transitions
- Bump pop has scale overshoot and flash
- Width/speed/force boost visuals are distinct
- All existing tests pass
