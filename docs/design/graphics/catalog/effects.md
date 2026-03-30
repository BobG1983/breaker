# Effects

## Combat Effects (Triggered)

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Shockwave | NONE | Critical | High | High | COVERED | Entity with expansion logic, damage in radius. No visual ring, no distortion. |
| Chain Lightning | NONE | Critical | High | High | COVERED | Arc entities track targets + damage. No visual arcs rendered. |
| Piercing Beam | NONE | Critical | High | High | COVERED | Beam entities process damage. No visible beam line. |
| Pulse | NONE | Important | High | High | COVERED | Emitter + ring expansion. No visual filled circle or distortion. |
| Explode | NONE | Critical | High | High | COVERED | Instant area damage. No flash, no burst, no particles. |
| Gravity Well | NONE | Critical | High | High | COVERED | Pull force entities. No distortion lens, no radius indicator. |
| Tether Beam | NONE | Critical | High | High | COVERED | Beam damage between bolts. No visible beam line. |
| Attraction/Magnetism | NONE | Low | Low-Med | Medium | COVERED | Faint curved arcs (<0.3 HDR) between bolt and target. Wake trail bends toward target. Brighten at close range. Minimal by design. |
| Ramping Damage (Amp) | NONE | Low | Low-Med | Medium | COVERED | Halo shifts warmer with consecutive hits (base→amber→white-hot). Orbiting energy ring spins faster. On whiff reset: heat drains, ring shatters. |
| Random Effect (Flux) | NONE | Low | Medium | Medium | COVERED | Brief prismatic flash (~0.1s, 3-4 spectral colors) then resolves to selected effect's visual. Multi-colored spark starburst. Fast and subtle. |

## Defensive / Utility Effects

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Shield (bolt-loss) | NONE | Critical | High | High | COVERED | `ShieldActive` charges are logic-only. No bottom barrier. |
| Second Wind wall | NONE | Important | High | Medium | COVERED | Invisible wall, no materialization flash. |
| Quick Stop | NONE | Low | Low | Low | COVERED | Aura compresses forward (squash, 2-3 frames). Spark spray forward from leading edge. Trail abruptly terminates. Micro-distortion at high stacks. |
| Time Penalty | NONE | Low | Low-Med | Medium | COVERED | Timer red-orange flash + glitch (chromatic split, jitter, 2-3 frames). Red-orange energy line from source to timer. Single danger vignette pulse. |
| Life Lost | NONE | Critical | High | High | COVERED | Decrements lives silently. Style guide: slow-mo + vignette. |

## Visual Modifiers (Active on Bolt/Breaker)

| Element | Status | Readability | Juice | Cohesion | Style Guide | Current |
|---------|--------|-------------|-------|----------|-------------|---------|
| Speed Boost (bolt) | NONE | Important | Medium | High | COVERED | Modifies speed. No trail extension, no glow change. |
| Damage Boost (bolt) | NONE | Important | Medium | High | COVERED | Modifies multiplier. No color shift, no brightness change. |
| Piercing (bolt state) | NONE | Important | Medium | High | COVERED | Bolt passes through. No angular glow, no energy spikes. |
| Size Boost (bolt) | PARTIAL | Low | Low | Low | COVERED | Scale changes. No proportional glow scaling. |
| Size Boost (breaker) | PARTIAL | Low | Low | Low | COVERED | Width scales. No aura stretch. |
| Bump Force Boost | NONE | Low | Low | Medium | COVERED | On bump: concentrated flash scaled by multiplier. At 2x+: compact radial ring from impact. At 3x+: HDR >2.0 bloom at impact. Distinct from Shockwave (compact, not expanding). |
| Visual Modifier System | NONE | Important | High | High | COVERED | Diminishing-returns stacking system. Not implemented. |
