# 5t: Evolution VFX Batch 1 — Beams

**Goal**: Implement bespoke VFX for the beam-type evolutions. These are crown jewels — they must look fundamentally different from base chip effects.

## **DECISION REQUIRED: DR-9 (partial)**

VFX directions exist for both evolutions in this batch. DR-9 may refine details.

## Evolutions

### 1. Nova Lance

**VFX direction**: Massive beam — full-screen-height piercing laser.

- Full-height beam from bolt through all cells in line
- Heavy bloom (HDR >2.0) with glow spilling into surrounding space
- Screen distortion along beam path (radial distortion from 5d)
- Beam appears near-instantly, holds for ~0.2s, then fades with afterimage
- Chromatic aberration pulse on fire (from 5k)
- Medium screen shake on fire
- Distinct from base Piercing Beam (5m) — wider, more intense, full-screen height

### 2. Railgun

**VFX direction**: Thin hyper-bright beam, instantaneous, no travel time.

- Razor-thin beam across entire screen width/height
- Hyper-bright core (HDR >3.0) — the brightest single element in the game
- Chromatic aberration trail along beam path
- No travel time — appears fully formed in a single frame
- Brief screen flash (white) on fire
- Small directional shake along beam axis
- Distinct from Nova Lance — thinner, brighter, more instantaneous

### Shared Beam Infrastructure

Both beams share rendering infrastructure:
- Beam entity with start/end positions, width, HDR intensity
- Beam Material2d shader (bright core + bloom + optional distortion)
- Afterimage fade system (beam lingers as fading ghost)

Build this shared infrastructure, then specialize per evolution.

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing: bloom, distortion, chromatic aberration), 5k (screen effects: shake, flash), 5m (base Piercing Beam VFX as reference — evolution beams should feel like a tier above)

## Catalog Elements Addressed

From `catalog/evolutions.md`:
- Nova Lance: NONE → bespoke VFX
- Railgun: NONE → bespoke VFX

## Verification

- Nova Lance beam spans full screen height with heavy distortion
- Railgun beam is thinner, brighter, and instantaneous
- Both are visually distinct from base Piercing Beam (5m)
- Screen effects (shake, flash, chromatic aberration) fire correctly
- Beam afterimages fade smoothly
- All existing tests pass
