---
name: Adversarial Quality Patterns
description: Techniques that find real bugs in this codebase — and anti-patterns to avoid when reviewing scenarios
type: feedback
---

## What works well (confirmed in existing scenarios)

- **Stress + seed diversity**: `stress: (runs: 32, parallelism: 32)` with different seeds explores probability space that single runs miss. Especially valuable for RandomEffect and any randomized spawn direction.
- **Rapid trigger + BoltSpeedInRange**: High action_prob (0.9) + SpeedBoost per trigger quickly reveals clamping failures.
- **NoEntityLeaks on spawn effects**: SpawnBolts/ChainBolt/Shockwave under dense layout + rapid destruction is the canonical pattern for entity lifecycle gaps.
- **4-deep trigger chains (supernova_chain_stress)**: Tests the arm/resolve cycle across multiple event depths — structural stress that unit tests cannot replicate.
- **Until + expiry reversal**: overclock_until_speed correctly uses 5000 frames to accumulate many Until cycles. This is the right pattern for time-expiry reversal bugs.
- **Slow arc_speed for mid-flight destruction**: chain_lightning_arc_lifecycle uses arc_speed: 50.0 deliberately so arcs are in-flight when targets are destroyed. This pattern applies to any "entity in transit" cleanup scenario.

## Anti-patterns to flag in scenarios

- **No invariants that verify correct behavior** — scenarios that only list `BoltInBounds, NoNaN` don't verify the effect did anything. A SpeedBoost scenario that doesn't include `BoltSpeedInRange` tells you it didn't crash, not that it worked.
- **Too-short max_frames for timer effects**: A GravityWell or Shield scenario with 500 frames may never complete a duration cycle. Effects with timers need at least 3000–5000 frames.
- **Weak layout for area effects**: Shockwave/Explode/ChainLightning at Corridor layout may never encounter cells near the bolt path. Dense or Scatter are better for area effect coverage.
- **Passive effects without reversal test**: DamageBoost/SpeedBoost/Piercing scenarios should use an Until wrapper to exercise the reversal path, not just accumulation.
- **Single-trigger scenarios for effects with type deactivation**: Attraction deactivates on hitting the target type and reactivates on bouncing off non-attracted types — this requires multi-trigger scenarios (Impacted(Cell), Impacted(Wall)) to exercise the full cycle.
- **Prism scenarios without BoltCountReasonable**: All Prism scenarios need BoltCountReasonable with an explicit max_bolt_count — otherwise unlimited accumulation goes unchecked.
- **AllBolts/AllCells targets without an invariant that proves all entities received the effect**: If an effect targets AllBolts but only 1 of 3 bolts gets it, no invariant catches this without a specific count-based check.

## When a scenario tests current behavior instead of desired behavior

- ramping_damage_reset uses `NoNaN, BoltInBounds` — these don't verify RampingDamage actually accumulated correctly. The scenario documents "it doesn't crash" not "it works as designed." A RunStatsMonotonic or dedicated accumulated-damage check would prove correct behavior.
- damage_boost_until_reversal uses `BoltSpeedInRange, NoNaN` — neither invariant directly detects an incorrect DamageMultiplier. There's no `EffectiveDamageConsistent` equivalent.
- chain_lightning_chaos: previously missing ChainArcCountReasonable (now fixed). Lesson: new invariants must be backfilled into existing related scenarios immediately.

## How to apply

When reviewing a proposed scenario for completeness, check:
1. Does it have an invariant that would fail if the effect didn't fire? (not just NoNaN)
2. Is max_frames long enough for multiple timer cycles?
3. Does the layout guarantee the effect has targets to hit?
4. If the effect has reversal, is the reversal path exercised?
5. If using AllBolts/AllCells, is there an invariant that proves all entities received the effect?
6. If the chip uses BumpWhiff/NoBump/Death/Died/NodeEnd triggers, these are fully uncovered — flag as HIGH gap.
