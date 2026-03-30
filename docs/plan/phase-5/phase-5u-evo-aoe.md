# 5u: Evolution VFX Batch 2 — AoE

**Goal**: Implement bespoke VFX for area-of-effect evolutions. Screen-filling spectacle.

## **DECISION REQUIRED: DR-9 (partial)**

VFX directions exist for all three. Dead Man's Hand redesign may still be pending.

## Evolutions

### 1. Supernova

**VFX direction**: Screen-filling explosion.

- Radial burst from center — expanding light ring fills entire playfield
- White-hot frame (HDR >3.0, 1-2 frames) — everything briefly washes out
- Then resolving — ring continues outward past screen edges
- Heavy screen shake
- Dense Spark particle spray from center
- Chromatic aberration pulse
- All cells in radius flash before destruction

### 2. Gravity Well (Evolution)

**VFX direction**: Larger/more intense version of base Gravity Well (5m).

- Wider radius than base gravity well
- Stronger screen distortion — visible background warping
- More intense rotation animation
- Additional Glow Mote particle density
- Evolution tier = the distortion is powerful enough to noticeably warp nearby entities visually (even though gameplay positioning is unchanged)
- Energy Ring pulse on activation

### 3. Dead Man's Hand

**VFX direction**: All bolts pulse simultaneously — synchronized shockwave rings.

- All active bolts simultaneously emit expanding shockwave rings
- Rings are synchronized (start at same frame)
- Each ring follows the shockwave VFX from 5m but with evolution-tier intensity
- Overlapping rings create white-hot zones (additive blending from 5d)
- Medium screen shake
- Brief slow-mo on trigger (~0.1s) — a "time freezes, then BOOM" feel

Note: design may be revised pending DR-9 resolution. Current implementation should be flexible.

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing: bloom, distortion), 5e (particles: Spark, Glow Mote, Energy Ring), 5k (screen effects: shake, flash, slow-mo), 5m (base Shockwave and Gravity Well VFX as reference)

## Catalog Elements Addressed

From `catalog/evolutions.md`:
- Supernova: NONE → bespoke VFX
- Gravity Well (evo): NONE → bespoke VFX
- Dead Man's Hand: NONE → bespoke VFX

## Verification

- Supernova fills the screen with a white-hot burst
- Gravity Well evolution is visibly more intense than base
- Dead Man's Hand produces synchronized rings from all bolts
- Additive blending creates visible intensity in overlap zones
- Screen effects fire correctly for each evolution
- All existing tests pass
