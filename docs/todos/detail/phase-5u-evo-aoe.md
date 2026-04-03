# 5u: Evolution VFX Batch 2 — AoE

**Goal**: Implement bespoke VFX for area-of-effect evolutions.

## Evolutions

### Supernova

**Behavior**: Chain reaction — perfect bump → cell destroy → spawn 2 bolts + shockwave. The spectacle comes from cascading overlapping effects, NOT a single authored blast.

**VFX direction**: Subtle marker on Supernova-triggered effects.
- Base shockwave/bolt-spawn VFX from 5m fire normally
- Shockwaves triggered by Supernova chain get a visual distinction: brighter ring, extra spark density
- The screen fills up organically from many overlapping effects through additive blending — not from a single burst
- No bespoke "supernova explosion" — the evolution's visual identity is the emergent cascade density

### Gravity Well (Evolution)

**Behavior**: Cell destroy → gravity well (strength 500, 5s duration, 160 radius, max 4 active).

**VFX direction**: Larger/more intense version of base gravity well (5m).
- Wider radius than base gravity well
- Stronger screen distortion — visible background warping
- More intense rotation animation
- More glow mote particle density
- Evolution tier = distortion powerful enough to noticeably warp nearby entities visually (gameplay positioning unchanged)
- Energy ring pulse on activation

### Dead Man's Hand

**Behavior**: Bolt loss → shockwave + speed boost. NOTE: Mechanic needs a bigger payoff rethink — current effect is underwhelming for an evolution. Design deferred to Phase 7. VFX for this evolution should be designed alongside the mechanic rework.

**VFX direction (provisional, pending mechanic rework)**:
- Dramatic shockwave centered on where the bolt was lost
- Visual speed-up effect on remaining bolts (trails flare, glow intensifies)
- "Fury upon loss" fantasy — the loss triggers something powerful
- VFX will be finalized when the mechanic is redesigned

## Dependencies

- **Requires**: 5c (rantzsoft_vfx crate), 5d (post-processing: bloom, distortion), 5e (particles: Spark, Glow Mote, Energy Ring), 5k (screen effects: shake, flash, slow-mo), 5m (base Shockwave and Gravity Well VFX as reference)
- DR-9 resolved: Supernova corrected (cascade, not single blast), Dead Man's Hand pending mechanic rework

## Verification

- Supernova-triggered shockwaves are visibly distinct from base shockwaves (brighter, more sparks)
- Gravity Well evolution is visibly more intense than base
- Dead Man's Hand fires provisional VFX on bolt loss (pending rework)
- Additive blending creates visible intensity in overlap zones
- Screen effects fire correctly for each evolution
- All existing tests pass
