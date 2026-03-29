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

## Anti-patterns to flag in scenarios

- **No invariants that verify correct behavior** — scenarios that only list `BoltInBounds, NoNaN` don't verify the effect did anything. A SpeedBoost scenario that doesn't include `BoltSpeedInRange` tells you it didn't crash, not that it worked.
- **Too-short max_frames for timer effects**: A GravityWell or Shield scenario with 500 frames may never complete a duration cycle. Effects with timers need at least 3000–5000 frames.
- **Weak layout for area effects**: Shockwave/Explode/ChainLightning at Corridor layout may never encounter cells near the bolt path. Dense or Scatter are better for area effect coverage.
- **Passive effects without reversal test**: DamageBoost/SpeedBoost/Piercing scenarios should use an Until wrapper to exercise the reversal path, not just accumulation.
- **Single-trigger scenarios for effects with type deactivation**: Attraction deactivates on hitting the target type and reactivates on bouncing off non-attracted types — this requires multi-trigger scenarios (Impacted(Cell), Impacted(Wall)) to exercise the full cycle.

## How to apply

When reviewing a proposed scenario for completeness, check:
1. Does it have an invariant that would fail if the effect didn't fire? (not just NoNaN)
2. Is max_frames long enough for multiple timer cycles?
3. Does the layout guarantee the effect has targets to hit?
4. If the effect has reversal, is the reversal path exercised?
