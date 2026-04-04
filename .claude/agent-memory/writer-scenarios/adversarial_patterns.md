---
name: Adversarial patterns by mechanic
description: Effective adversarial techniques per effect/system domain — what actually stresses each mechanic
type: reference
---

## General adversarial principles that work

1. **Spawn-then-despawn racing**: effects that spawn entities (Shockwave, PiercingBeam, ChainLightning, TetherBeam, ChainBolt) under high-frequency triggers. Dense layout + chaos 0.6 maximises simultaneous spawns. Invariants: NoEntityLeaks + NoNaN.

2. **Until reversal accumulation**: wrap stat effects in Until(TimeExpires(N)) on a frequently-firing trigger. If reversal fails, stat accumulates unboundedly. Invariants: BoltSpeedAccurate (for SpeedBoost), NoNaN.

3. **Multiplier stacking detection**: put the same SpeedBoost/DamageBoost on multiple independent triggers that could fire simultaneously (EarlyBumped + PerfectBumped + LateBumped). If the bridge incorrectly fires multiple variants, speed blows past max. Invariants: BoltSpeedAccurate.

4. **Wall bounce amplification**: Corridor layout + impacted(Wall) trigger + SpeedBoost is the canonical "does clamping hold?" test. Every wall bounce fires the effect. 8000 frames gives hundreds of applications.

5. **Edge clamping stress**: BreakerPositionClamped + BreakerInBounds + chaos 0.8-0.9 + Corridor layout for breaker effects (QuickStop). Rapid direction reversals near walls are the adversarial condition.

6. **NodeTimerThreshold replay risk**: If threshold fires more than once (timer rebounds across 0.5 due to float imprecision), TimePenalty compounds. Use Chrono breaker + Dense layout + TimerNonNegative. Do NOT include TimerMonotonicallyDecreasing with TimePenalty (penalty causes valid downward jumps that look like non-monotone behavior to that invariant).

7. **NodeStart double-fire**: SpeedBoost on NodeStart at high multiplier (1.5x). If NodeStart fires twice per node, bolt hits 2.25x base speed and escapes BoltMaxSpeed. Use BoltSpeedAccurate.

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
| Speed integrity | BoltSpeedAccurate + BoltInBounds + NoNaN |
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

## Global trigger + AllBolts binding pattern

Global triggers (BumpWhiff, Death, Impact(Breaker), NodeStart, NodeEnd, BoltLost) fire on
ALL entities with BoundEffects, not just bolts. To bind a global trigger effect to the bolt:
use `On(target: AllBolts, then: [When(trigger: GlobalTrigger, ...)])`. This installs the
When node into each bolt's BoundEffects. When the trigger fires globally, each bolt evaluates
its own When node and fires the effect on itself.

This pattern is correct and validated for: BumpWhiff, Death, Impact(Breaker), NodeStart.

## Death trigger adversarial risk (Dense layout)

Dense layout + `When(trigger: Death, ...)` with a stat multiplier effect is the canonical
stress test for multi-cell destruction. In a single-tick pierce pass through N cells,
Death fires N times, compounding SpeedBoost(1.05) to 1.05^N per tick. Use a small
multiplier (1.05) so single-fire is safe but double-fire accumulates detectably.
BoltSpeedAccurate is the primary detector.

## Impact(Breaker) + Corridor accumulation

Corridor layout + `When(trigger: Impact(Breaker), ...)` + small SpeedBoost (1.05x) is
the canonical "does double-fire on breaker contact accumulate?" test. The narrow channel
forces high-frequency breaker contacts. 8000 frames at 64 Hz = ~500+ contacts.
1.05^500 is astronomical — any double-fire is immediately detectable via BoltSpeedAccurate.

## BumpWhiff with Once + AOE combined

`When(trigger: BumpWhiff, then: [Once([Do(DamageBoost(N))]), Do(Shockwave(...))])`
stress-tests two independent dispatch paths in one whiff event: Once (consumed on first
match) and an unconditional AOE spawn. If the bridge double-reads the message, Once fires
twice (consuming and then silently failing) while Shockwave spawns two rings per whiff.
Use NoEntityLeaks to catch the ring accumulation and BoltSpeedAccurate for DamageBoost leakage.

## Seed selection

Use distinct primes or memorable numbers per scenario to avoid identical RNG paths:
- 5513, 7722, 3377, 1984, 6174, 8191, 2357, 1024, 4096, 6561
- Avoid seeds already used: 0, 42, 200, 1337, 2718, 3141, 4242, 7331, 8080, 9999
- chaos/ scenario seeds used: 2311 (whiff), 1301 (node_end), 4271 (death), 5003 (impact_breaker)
