# 5e: Particle System

**Goal**: Build `rantzsoft_particles` — a custom 2D GPU particle system crate — and implement the 6 core particle types that all later VFX steps will use.

## Decision: Custom Crate (`rantzsoft_particles`)

Evaluated bevy_hanabi (macOS pink screen bug), bevy_enoki (no additive blending, CPU-only), and others. All had disqualifying issues. Building a custom crate gives full control over the rendering pipeline, additive blending, HDR bloom interaction, and avoids external dependency risk.

`rantzsoft_particles` follows the existing rantzsoft_* convention: game-agnostic, Bevy plugin, zero game vocabulary.

## What to Build

### 1. `rantzsoft_particles` Crate

New workspace member at `rantzsoft_particles/` with:
- HDR color support (particles need to bloom)
- License, maintenance status, binary size

If no crate meets requirements, build a lightweight custom system.

- `RantzParticlesPlugin` — Bevy plugin
- GPU compute shader for particle simulation (position, velocity, lifetime, color, size)
- Custom `Material2d` with `AlphaMode::Add` for additive blending
- Buffer management for GPU-side particle state
- Emitter component: configurable spawn rate, burst mode, one-shot mode
- Color over lifetime: `Gradient<LinearRgba>` with HDR support (values >1.0 for bloom)
- Size over lifetime: `Gradient<f32>`
- Emission shapes: point, circle, line
- RON-serializable configuration for all particle parameters

### 2. Integration with rendering/

Wire `rantzsoft_particles` into the rendering/ domain:
- rendering/ depends on `rantzsoft_particles` (like it depends on `rantzsoft_spatial2d`)
- Particle emitter management systems
- Integration with the additive blending pipeline from 5d
- Integration with temperature palette (5f) for color tinting

### 3. Implement 6 Particle Types

Each particle type is a reusable building block used by many later VFX steps:

| Type | Shape | Behavior | Used By (later steps) |
|------|-------|----------|----------------------|
| **Spark** | Point/tiny streak | Burst outward, fade quickly, slight gravity | Cell destruction (5i), bump feedback (5l), impact effects |
| **Trail** | Elongated streak | Follows emitter, fades with distance | Bolt wake (5g), dash trail (5h), beam afterimage (5m) |
| **Shard** | Small angular fragment | Burst outward with rotation, slower fade | Cell shatter (5i), shield break (5l) |
| **Glow mote** | Soft circle | Drifts slowly, long lifetime, ambient | Background sprites (5j), gravity well ambient (5m) |
| **Energy ring** | Expanding circle | Expands and fades | Shockwave (5m), pulse (5m), bump feedback (5l) |
| **Electric arc** | Jagged line segment | Flickers rapidly, short lifetime | Chain lightning (5m), electric effects (5w) |

Each type needs:
- Configurable color (base + temperature tint)
- Configurable HDR intensity (for bloom interaction)
- Configurable count, lifetime, velocity, size
- Additive blending

### 4. Debug Visualization

Add debug menu controls:
- Particle count display
- Per-type spawn test buttons (fire a burst of each type)
- Performance overlay showing active particle count

## What NOT to Do

- Do NOT implement specific VFX that use particles (those are 5i, 5l, 5m, etc.)
- Do NOT implement particle density scaling yet (that emerges naturally from build complexity)
- Just build the types and prove they work via debug triggers

## Dependencies

- **Requires**: 5c (rendering/ domain exists)
- **Independent of**: 5d (post-processing pipeline) — can be done in either order
- **Enhanced by**: 5d (additive blending, bloom) — particles look better with post-processing but work without it

## Catalog Elements Addressed

From `catalog/feedback.md` (Particle Types section):
- Spark particles: NONE → implemented
- Trail particles: NONE → implemented
- Shard particles: NONE → implemented
- Glow mote particles: NONE → implemented
- Energy ring particles: NONE → implemented
- Electric arc particles: NONE → implemented

## Verification

- All 6 particle types spawn correctly via debug menu
- Particles use additive blending (if 5d is done)
- Particles bloom with HDR intensity (if 5d is done)
- Particle count stays reasonable (no leaks)
- All existing tests pass
