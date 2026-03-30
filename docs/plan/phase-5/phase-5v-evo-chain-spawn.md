# 5v: Evolution VFX Batch 3 — Chain/Spawn

**Goal**: Implement bespoke VFX for evolutions that involve chain reactions, entity spawning, and recursive effects.

## **DECISION REQUIRED: DR-9 (partial)**

VFX directions exist for all four evolutions in this batch.

## Evolutions

### 1. Chain Reaction

**VFX direction**: Recursive spawn with escalating visual intensity.

- Each recursive spawn: expanding Energy Ring from destroyed cell
- Bright energy lines flash between destroyed cell and spawned bolts ("inheritance streamers")
- Additive blending creates white-hot cascade zones when multiple chain reactions overlap
- Chromatic aberration per generation tier (1st gen = subtle, 3rd gen = intense)
- Screen shake scales with generation depth

### 2. Split Decision

**VFX direction**: Cell fission effect.

- On trigger: target cell's glow splits along a visible axis
- Halves condense into bolt-shaped orbs (~0.15t)
- Energy filaments connect the two halves during split
- Spawned bolts inherit parent bolt's visual modifiers (from 5n)
- Prismatic Trail particles on spawned bolts briefly
- Brief flash at split moment

### 3. Feedback Loop

**VFX direction**: Three-node triangle charge indicator with circuit-close detonation.

- Persistent: Three-node triangle indicator near bolt, rendered as faint connected dots
- Charge: Each perfect bump lights a node (dim → bright)
- On completion: All three nodes flash white-hot (HDR >1.5), collapse inward, circuit closes
- Spawned bolt + shockwave fire with amplified VFX (larger than base shockwave)
- Screen flash + medium shake on circuit close
- The charge phase is subtle; the payoff is dramatic

### 4. Entropy Engine

**VFX direction**: Prismatic mote orbit gauge with burst detonation.

- Persistent: Ring of prismatic Glow Mote particles orbiting the bolt as a counter gauge
- Motes grow in number/density as counter fills (across the node, not per hit)
- On counter fill: motes converge toward bolt center, then detonate in multi-colored burst
- Burst: HDR >2.0, chromatic aberration pulse, Spark spray in all spectral colors
- Mote ring grows denser across the node (visual escalation within a single node)

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing: bloom, chromatic aberration), 5e (particles: Energy Ring, Spark, Trail, Glow Mote), 5k (screen effects: shake, flash), 5m (base Shockwave VFX for Feedback Loop payoff), 5n (visual modifier inheritance for Split Decision)

## Catalog Elements Addressed

From `catalog/evolutions.md`:
- Chain Reaction: NONE → bespoke VFX
- Split Decision: NONE → bespoke VFX
- Feedback Loop: NONE → bespoke VFX
- Entropy Engine: NONE → bespoke VFX

## Verification

- Chain Reaction shows escalating intensity with generation depth
- Split Decision shows visible cell fission with energy filaments
- Feedback Loop has persistent charge indicator that pays off dramatically
- Entropy Engine has orbiting motes that converge and detonate
- All spawn-related VFX correctly track new entities
- All existing tests pass
