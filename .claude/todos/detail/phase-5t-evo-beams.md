# 5t: Evolution VFX Batch 1 — Beams

**Goal**: Implement bespoke VFX for the beam evolution. Crown jewel — must look fundamentally different from base chip effects.

## Evolution

### Nova Lance

**Behavior**: PiercingBeam on perfect bump + cell impact.
NOTE: Mechanic currently uses Shockwave in the RON file — needs updating to PiercingBeam. If mechanic change hasn't happened by this step, VFX should still be a beam (design intent) and will connect to the mechanic when it's updated.

**VFX direction**: Massive beam along bolt's trajectory.
- Beam appears at max width, fully formed
- Width shrinks over a short but noticeable duration (not instant — the beam lingers)
- Heavy bloom (HDR >2.0) with glow spilling into surrounding space
- Screen distortion along beam path (radial distortion from 5d)
- Chromatic aberration pulse on fire (from 5k)
- Medium screen shake on fire
- Distinct from base Piercing Beam (5m) — thicker, more intense, longer duration, width-shrink animation

### Beam Infrastructure

Beam rendering infrastructure for Nova Lance (and any future beam effects):
- Beam entity with start/end positions, width, HDR intensity
- Beam Material2d shader (bright core + bloom + optional distortion)
- Width-over-time animation (starts at max, shrinks to zero)
- Afterimage fade system (beam lingers as fading ghost)

Nova Lance is the sole beam evolution.

## Dependencies

- **Requires**: 5c (rantzsoft_vfx crate), 5d (post-processing: bloom, distortion, chromatic aberration), 5k (screen effects: shake, flash), 5m (base Piercing Beam VFX as reference — evolution beam is a tier above)
- DR-9 resolved: Nova Lance VFX corrected to match beam fantasy

## Verification

- Nova Lance beam spans bolt trajectory with heavy distortion
- Beam appears at max width and shrinks over time (not instant)
- Visually distinct from base Piercing Beam (5m) — clearly an evolution-tier effect
- Screen effects (shake, flash, chromatic aberration) fire correctly
- Beam afterimage fades smoothly
- All existing tests pass
