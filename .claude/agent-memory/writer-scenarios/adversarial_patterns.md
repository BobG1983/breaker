---
name: Adversarial patterns by mechanic
description: Effective adversarial techniques per effect/system domain — what actually stresses each mechanic
type: reference
---

## General adversarial principles that work

1. **Spawn-then-despawn racing**: effects that spawn entities (Shockwave, PiercingBeam, ChainLightning, TetherBeam, ChainBolt) under high-frequency triggers. Dense layout + chaos 0.6 maximises simultaneous spawns. Invariants: NoEntityLeaks + NoNaN.

2. **Until reversal accumulation**: wrap stat effects in Until(TimeExpires(N)) on a frequently-firing trigger. If reversal fails, stat accumulates unboundedly. Invariants: BoltSpeedInRange (for SpeedBoost), NoNaN.

3. **Multiplier stacking detection**: put the same SpeedBoost/DamageBoost on multiple independent triggers that could fire simultaneously (EarlyBumped + PerfectBumped + LateBumped). If the bridge incorrectly fires multiple variants, speed blows past max. Invariants: BoltSpeedInRange.

4. **Wall bounce amplification**: Corridor layout + impacted(Wall) trigger + SpeedBoost is the canonical "does clamping hold?" test. Every wall bounce fires the effect. 8000 frames gives hundreds of applications.

5. **Edge clamping stress**: BreakerPositionClamped + BreakerInBounds + chaos 0.8-0.9 + Corridor layout for breaker effects (QuickStop). Rapid direction reversals near walls are the adversarial condition.

6. **NodeTimerThreshold replay risk**: If threshold fires more than once (timer rebounds across 0.5 due to float imprecision), TimePenalty compounds. Use Chrono breaker + Dense layout + TimerNonNegative. Do NOT include TimerMonotonicallyDecreasing with TimePenalty (penalty causes valid downward jumps that look like non-monotone behavior to that invariant).

7. **NodeStart double-fire**: SpeedBoost on NodeStart at high multiplier (1.5x). If NodeStart fires twice per node, bolt hits 2.25x base speed and escapes BoltMaxSpeed. Use BoltSpeedInRange.

## Layout selection guide

| Layout | Best for |
|--------|---------|
| Dense | Maximising simultaneous cell hits, spawn/despawn races, arc effects |
| Scatter | Irregular timing between impacts, varied gap widths, tether distance tests |
| Corridor | Wall bounce accumulation, breaker edge tests, speed clamp tests |
| Fortress | Long node duration, timer-sensitive effects, slow cell destruction |

## Invariant selection by adversarial goal

| Goal | Invariants |
|------|-----------|
| Entity lifecycle (spawn/despawn) | NoEntityLeaks + NoNaN |
| Speed integrity | BoltSpeedInRange + BoltInBounds + NoNaN |
| Breaker integrity | BreakerInBounds + BreakerPositionClamped + NoNaN |
| Timer integrity | TimerNonNegative (never TimerMonotonicallyDecreasing with TimePenalty) |
| Bolt count bounds | BoltCountReasonable (raise max_bolt_count if effect legitimately spawns multiple) |

## Stress scenario sizing

- 16 runs = sufficient for seed-sensitive arc/scatter effects
- 32 runs = default for entity lifecycle tests (shockwave, spawns)
- 8000 frames = maximum for dense effects; 5000 for mechanic scenarios; 3000 for single-trigger effects

## max_bolt_count guidelines

- Default (8) sufficient for: single-bolt effects, stat-only effects
- 12 = reasonable for: ChainBolt, SpawnBolts(count: 1), limited chain effects
- 16 = for: TetherBeam (spawns 2 per bump), stacking spawn effects
- 20+ = for: deep chain spawns (Supernova pattern)

## Seed selection

Use distinct primes or memorable numbers per scenario to avoid identical RNG paths:
- 5513, 7722, 3377, 1984, 6174, 8191, 2357, 1024, 4096, 6561
- Avoid seeds already used: 0, 42, 200, 1337, 2718, 3141, 4242, 7331, 8080, 9999
