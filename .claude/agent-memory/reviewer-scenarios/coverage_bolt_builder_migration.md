---
name: Bolt Builder Migration Coverage Map
description: Scenario and invariant gaps from bolt builder migration — steering model for attraction/gravity_well, BoltAngleSpread (was BoltRespawnAngleSpread, deleted Wave 6), PrimaryBolt marker
type: project
---

## What Changed (feature/chip-evolution-ecosystem branch)

- `init_bolt_params` system deleted — bolt config params now stamped by builder via `config()` call
- `spawn_extra_bolt` function removed — bolt spawning now exclusively via `Bolt::builder()`
- Attraction and gravity well now use steering model: `velocity = (velocity + steering).normalize_or_zero()` then `apply_velocity_formula` — this normalizes before speed clamping, meaning the prior velocity direction is lost on every steering tick
- `BreakerReflectionSpread` renamed from something else (now used directly in `CollisionQueryBreaker` via `BreakerReflectionSpread`)
- `BoltSpeedInRange` invariant renamed to `BoltSpeedAccurate` — checks `(base * mult).clamp(min, max)` instead of just a range check

## Scenario Coverage Status

| Mechanic | Coverage | Quality |
|----------|----------|---------|
| Bolt builder primary path (spawn + node reset) | Implicit via all bolt-containing scenarios | Never explicitly probed |
| Bolt builder extra path (ExtraBolt, CleanupOnNodeExit) | prism_bolt_stabilization, spawn_bolts_stress, prism_* scenarios | Adequate — NoEntityLeaks catches lifecycle |
| BoltAngleSpread via definition() | Only unit tests (launch_bolt.rs) | NO scenario verifies angle-spread distribution |
| Steering model: normalize then apply_velocity_formula | attraction_cell_chaos (BoltSpeedAccurate), gravity_well_chaos (BoltSpeedAccurate) | Adequate for speed. Zero-vector NaN still possible when steering delta equals negative velocity |
| BoltSpeedAccurate (renamed from BoltSpeedInRange) | Self-test: bolt_speed_inaccurate.scenario.ron | Good — self-test still valid |
| BreakerReflectionSpread in collision | aegis_speed_bounce, all Aegis scenarios | Covered implicitly |
| PrimaryBolt marker correctness | No scenario | NOT COVERED |
| Builder effect inheritance (with_inherited_effects) | prism_bolt_stabilization uses inherit:true in SpawnBolts | Weak — no invariant proves inherited effects actually fire on spawned bolts |

## Gaps Introduced by This Migration

1. **Steering model: normalize_or_zero on zero-velocity bolt** — if both old velocity and steering delta cancel to zero, `normalize_or_zero` produces Vec2::ZERO, then `apply_velocity_formula` tries to constrain a zero vector. The result is speed 0 but bolt is not serving — bolt becomes permanently stuck. `BoltSpeedAccurate` skips zero-speed bolts, so this failure mode is invisible to all current invariants.

2. **BoltSpawnParams injection via definition() — no scenario probes component presence** — `BoltRespawnOffsetY`, `BoltRespawnAngleSpread`, and `BoltInitialAngle` were deleted in Wave 6. `LostBoltData` now queries `BoltSpawnOffsetY` and `BoltAngleSpread`. No invariant catches missing required bolt components on spawn.

3. **ExtraBolt cleanup correctness on node transition** — `ExtraBolt` entities use `CleanupOnNodeExit` but `PrimaryBolt` uses `CleanupOnRunEnd`. If a bug swapped these, primary bolts would be despawned on node exit. No invariant distinguishes primary vs extra lifecycle correctness.

## How to apply

- Flag zero-velocity stuck bolt as HIGH gap when steering is active (attraction or gravity well) — the scenario should use `BoltSpeedAccurate` AND verify bolt count doesn't drop unexpectedly.
- Flag missing PrimaryBolt marker scenario as MEDIUM.
- Flag inherited effects on spawned bolts as already-known MEDIUM (pre-existing gap).
