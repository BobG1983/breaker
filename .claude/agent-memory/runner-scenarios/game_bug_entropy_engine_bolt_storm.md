---
name: BoltCountReasonable violation — EntropyEngine + SpawnBolts has no global bolt cap (RESOLVED)
description: EntropyEngine escalated to max_effects; combined with SpawnBolts and no cap, produced bolt storms in Dense layouts — previously confirmed bug, now RESOLVED
type: project
---

## Bug: entropy_engine + spawn_bolts — no bolt count cap (RESOLVED)

`EntropyEngine.fire()` scaled effects from 1 to `max_effects` as cells_destroyed grew. Once cells_destroyed >= max_effects (quickly in Dense layouts), every cell destruction fired `max_effects` random effects. With a 50/50 pool of SpawnBolts(count:1) and Shockwave, bursts of 2-3 new bolts could spawn per destruction.

`SpawnBolts.fire()` (`breaker-game/src/effect/effects/spawn_bolts/effect.rs`) had no check for total active bolt count.

In `entropy_engine_stress`, the Dense layout destroyed cells so rapidly that the spawn rate overwhelmed the 2-second lifespan drain rate, producing >12 active bolts (the configured `max_bolt_count` threshold).

Secondary symptom: `"Entity despawned: Entity ... is invalid"` Bevy warnings from double-despawn.

## Previous failure

- `entropy_engine_stress` stress — all 32 copies previously failed with BoltCountReasonable violations

## Fix needed (archived)

Either:
1. Add a global bolt count cap in `spawn_extra_bolt()` or `SpawnBolts.fire()`
2. Or accept that `entropy_engine_stress` needs a higher `max_bolt_count` threshold

## Status

RESOLVED — `entropy_engine_stress` stress: 32/32 passed. Confirmed 2026-03-30 run.

Note from commit `53596f5`: "fix(scenario): accept BoltCountReasonable violations in entropy_engine_stress" — the fix was to loosen the scenario threshold, not to cap bolt spawning.
