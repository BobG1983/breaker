# 5v: Evolution VFX Batch 3 — Chain/Spawn

**Goal**: Implement bespoke VFX for evolutions that involve chain reactions, entity spawning, and recursive effects.

## Evolutions

### Chain Reaction

**Behavior**: Cell destroy → small shockwave. Shockwave kills → another shockwave (recursive, max depth capped). NOTE: No RON file yet — mechanic needs implementing in Phase 7.

**VFX direction**: Recursive shockwaves with escalating visual intensity.
- Each recursive shockwave: expanding energy ring from destroyed cell
- Visual intensity escalates with generation depth (1st gen = base, 2nd = brighter, 3rd = intense)
- Chromatic aberration scales with generation depth
- Screen shake scales with total chain size
- Ingredients: Chain Reaction x2 + Aftershock x2

### Split Decision

**Behavior**: Cell destroy → spawn 2 bolts with effect inheritance.

**VFX direction**: Cell fission effect.
- On trigger: target cell's glow splits along a visible axis
- Halves condense into bolt-shaped orbs (~0.15s)
- Energy filaments connect the two halves during split
- Spawned bolts inherit parent bolt's visual modifiers (from 5n)
- Prismatic trail particles on spawned bolts briefly
- Brief flash at split moment

### Feedback Loop

**Behavior**: Track N perfect bumps → spawn bolt + large shockwave on completion, reset counter. NOTE: No RON file yet — mechanic needs implementing in Phase 7.

**VFX direction**: Three-node triangle charge indicator with circuit-close detonation.
- Persistent: Three-node triangle indicator near bolt, rendered as faint connected dots
- Charge: Each perfect bump lights a node (dim → bright)
- On completion: All three nodes flash white-hot (HDR >1.5), collapse inward, circuit closes
- Spawned bolt + shockwave fire with amplified VFX (larger than base shockwave)
- Screen flash + medium shake on circuit close
- The charge phase is subtle; the payoff is dramatic
- Ingredients: Feedback Loop x2 + Bump Force x2

### Entropy Engine

**Behavior**: Cell destroy → random effect from weighted pool (SpawnBolts, Shockwave, ChainBolt, SpeedBoost). Fires on every cell destroy, no counter.

**VFX direction**: Prismatic flash per trigger (randomness visualized).
- Brief prismatic flash (~0.1s, 3-4 spectral color cycle) on each cell destroy, then resolves to the selected effect's visual
- Multi-colored spark starburst from entity on selection
- Bolt has a persistent prismatic shimmer while Entropy Engine is active (distinguishes it from a normal bolt)
- Fast and subtle per-trigger — Entropy Engine modifies other effects, the randomness is the identity
- No counter gauge (mechanic has no counter)

## Dependencies

- **Requires**: 5c (rantzsoft_vfx crate), 5d (post-processing: bloom, chromatic aberration), 5e (particles: Energy Ring, Spark, Trail, Glow Mote), 5k (screen effects: shake, flash), 5m (base Shockwave VFX for Chain Reaction and Feedback Loop payoff), 5n (visual modifier inheritance for Split Decision)
- DR-9 resolved: Entropy Engine corrected (no counter gauge, prismatic flash per trigger)

## Verification

- Chain Reaction shows escalating intensity with generation depth
- Split Decision shows visible cell fission with energy filaments
- Feedback Loop has persistent charge indicator that pays off dramatically
- Entropy Engine shows prismatic flash per trigger and bolt shimmer
- All spawn-related VFX correctly track new entities
- All existing tests pass
