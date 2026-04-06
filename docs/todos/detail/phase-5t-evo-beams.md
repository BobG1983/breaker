# 5o: Evolution VFX — Beams

## Summary

Implement bespoke VFX for the beam evolution (Nova Lance). This is the sole beam-type evolution and serves as the first crown-jewel evolution VFX. Nova Lance must look fundamentally different from the base Piercing Beam effect from 5l — thicker, more intense, longer duration, with a distinctive width-shrink animation. Beam rendering infrastructure built here also supports any future beam-style effects.

## Context

Evolution VFX are the visual payoff for maxing out chip combinations. They must feel like ultimate abilities — a tier above all base chip effects in visual complexity, particle density, and screen presence. Each evolution gets completely bespoke VFX authored as direct Rust functions (no recipe system, no RON visual sequences).

Key architecture changes from the LEGACY plan:
- **No recipe system.** Nova Lance VFX is a direct Rust function/system, not a RON recipe.
- **rantzsoft_particles2d** for particle effects (spark bursts, afterimage particles).
- **rantzsoft_postprocess** for screen effects (radial distortion, chromatic aberration, screen flash).
- **Visual types from visuals/ domain** (5e) — Hue, Shape, GlowParams, VisualModifier.
- **DR-9 correction**: Nova Lance's mechanic needs changing from Shockwave to PiercingBeam in Phase 7. The VFX should be a beam regardless of when the mechanic is updated — design intent is beam fantasy.

Current RON (`nova_lance.evolution.ron`):
- Trigger: PerfectBumped -> Impacted(Cell) -> `Do(PiercingBeam(damage_mult: 2.5, width: 40.0))`
- Ingredients: Impact x2 + Bolt Speed x3
- Note: RON already uses `PiercingBeam` effect, so the mechanic may have been updated since DR-9 was written.

## Evolutions in This Batch

### Nova Lance

**Mechanic summary**: On perfect bump + cell impact, fires a devastating piercing beam through all cells in the bolt's trajectory. Beam is wide (40.0) with 2.5x damage multiplier.

**VFX design (from DR-9 and effects-particles.md)**:
- Massive beam along the bolt's trajectory
- Beam appears at full max width instantly (not a thin beam that grows)
- Width shrinks over a short but noticeable duration (the beam lingers and narrows)
- Heavy bloom (HDR >2.0) with glow spilling into surrounding space
- Screen distortion along the beam path (radial distortion from rantzsoft_postprocess)
- Chromatic aberration pulse on fire
- Medium screen shake on fire
- Spark burst at beam origin and along the beam path
- Afterimage: beam lingers as a fading ghost after the primary beam fades
- Distinct from base Piercing Beam (5l): thicker, more intense bloom, longer duration, width-shrink animation, screen-level effects (distortion + chromatic aberration + shake)

**What to implement**:
1. Beam entity system: spawn a beam entity with start position, end position (bolt trajectory), width, HDR intensity, and lifetime
2. Beam Material2d shader: bright core line with bloom-friendly HDR values, optional per-pixel distortion
3. Width-over-time animation system: beam starts at max width (40.0), shrinks to zero over duration (~0.4s)
4. Afterimage fade system: after primary beam fades, a ghost beam at reduced intensity lingers (~0.2s)
5. Nova Lance fire function: called by the PiercingBeam effect's `fire()` when triggered by an evolution-tier chip. Spawns beam entity + triggers screen effects (distortion, chromatic aberration, screen shake, screen flash) + spawns spark burst particles at beam origin
6. Visual distinction logic: detect that the PiercingBeam was triggered by Nova Lance (via chip attribution) and route to the evolution-tier VFX instead of the base Piercing Beam VFX

## What to Build

### 1. Beam Entity and Components

Create beam-specific components in the `fx/` or `effect/` domain (wherever combat VFX from 5l are housed):

- `BeamEffect` component: start position, end position, initial width, current width, HDR intensity, lifetime timer, afterimage duration
- Beam entity bundle: `BeamEffect` + `Transform` + `Mesh2d` (stretched quad along beam axis) + `MeshMaterial2d<BeamMaterial>`

### 2. Beam Material (Material2d)

Custom shader for beam rendering:

- Bright core (narrow central line at full HDR)
- Bloom halo (wider soft glow around core, intensity falls off with distance from center)
- Additive blending via `specialize()` (same pattern as ParticleMaterial)
- Uniform inputs: width, intensity, core_color, halo_falloff
- Shader file location: within game crate's shader assets

### 3. Beam Animation Systems

- `animate_beam_width`: each tick, interpolate beam width from initial toward zero over lifetime. Update mesh scale and material uniform.
- `animate_beam_afterimage`: when primary beam lifetime expires, spawn (or transition to) afterimage state — reduced intensity, continued fade over afterimage duration.
- `cleanup_beam`: despawn beam entity when afterimage duration expires.

### 4. Nova Lance VFX Function

The evolution-specific VFX orchestrator. Called when PiercingBeam fires with Nova Lance attribution:

- Spawn beam entity along bolt trajectory (start = bolt position, end = trajectory endpoint)
- Trigger screen effects via rantzsoft_postprocess messages:
  - `TriggerRadialDistortion` along beam path (intensity ~0.4, duration ~0.4s)
  - `TriggerChromaticAberration` (intensity ~0.2, duration ~0.3s)
  - `TriggerScreenFlash` (color: White, intensity 1.0, duration ~3 frames)
- Trigger screen shake (medium tier)
- Spawn spark burst particles at beam origin via rantzsoft_particles2d (RadialBurst preset, ~20 particles, high HDR, short lifetime)

### 5. Evolution VFX Routing

Mechanism to distinguish base PiercingBeam VFX from Nova Lance VFX:

- When the PiercingBeam effect fires, check chip attribution (the `String` key in `BoundEffects`)
- If attributed to "Nova Lance", route to the Nova Lance VFX function
- If attributed to a base chip, route to the base Piercing Beam VFX (from 5l)
- This routing pattern will be reused by all evolution VFX phases

### 6. Tests

- Beam entity spawns with correct initial width and position
- Beam width shrinks toward zero over the configured lifetime
- Beam entity despawns after afterimage duration expires
- Nova Lance VFX function triggers all expected screen effects
- Evolution routing correctly identifies Nova Lance attribution
- Base PiercingBeam VFX still fires for non-evolution piercing shots

## What NOT to Do

- Do NOT implement the PiercingBeam mechanic change (Shockwave -> PiercingBeam in the RON) — that is Phase 7. The VFX connects to the existing PiercingBeam effect leaf.
- Do NOT create a recipe system or RON-driven VFX sequences. All VFX is direct Rust code.
- Do NOT modify base Piercing Beam VFX from 5l. Nova Lance is additive — it is a separate, higher-tier visual that fires instead of (not on top of) the base VFX.
- Do NOT implement other beam-type evolutions (none currently exist). Build infrastructure sufficient for Nova Lance; generalize only if future beams arrive.
- Do NOT add beam rendering to rantzsoft_particles2d or rantzsoft_postprocess. Beams are game-side visual entities, not crate-level primitives.

## Dependencies

- **5l** (combat effect VFX): base Piercing Beam VFX must exist as the reference point. Nova Lance must be visibly a tier above it. The VFX routing mechanism distinguishes base from evolution.
- **5c** (rantzsoft_particles2d): spark burst particles at beam origin.
- **5d** (rantzsoft_postprocess): screen distortion along beam path, chromatic aberration pulse, screen flash.
- **5e** (visuals/ domain): visual types (Hue, GlowParams) for beam color/intensity configuration.
- **5k** (bump/failure VFX): screen shake infrastructure.

## Verification

- Nova Lance beam spans the bolt's trajectory with heavy bloom and visible distortion
- Beam appears at max width and shrinks smoothly to zero over ~0.4s (not instant)
- Afterimage lingers and fades after the primary beam disappears
- Visually distinct from base Piercing Beam (5l) — clearly an evolution-tier effect (wider, brighter, longer, with screen effects)
- Screen effects (shake, flash, chromatic aberration, distortion) all fire on Nova Lance trigger
- Spark particles spawn at beam origin
- Base Piercing Beam VFX still works correctly for non-evolution piercing shots
- All existing tests pass

## Status: NEEDS DETAIL
