---
name: Scenario run status (develop, post-fad7dfa)
description: Last clean run was fad7dfa (103/103). After new scenarios added, boss_arena_chaos FAILS with AabbMatchesEntityDimensions game bug
type: project
---

## Last clean run: 2026-03-30 on commit fad7dfa

Branch: develop
Last commit: fad7dfa (Merge branch 'feature/missing-unit-tests' into develop)

## Current status (8 new scenarios added after fad7dfa)

1 FAIL: boss_arena_chaos — AabbMatchesEntityDimensions x19997 frames 0..20000

Root cause: game bug — breaker Aabb2D.half_extents is never updated when EntityScale is applied.
See game_bug_breaker_aabb_not_scaled.md for full diagnosis.

## Results

- 86 named scenario PASS (chaos, mechanic, self_test, and non-stress stress scenarios)
- 17 stress suites: all-passed
- 0 failures
- 0 violations in gameplay scenarios (self-test scenarios correctly fired their expected violations)
- Coverage Report: "All invariants have self-test coverage. All layouts are referenced."

## Runner output summary

scenario result: ok. 103 passed; 0 failed

## Previously known bugs — all resolved

1. BoltSpeedInRange — gravity_well ordering + enforce_distance_constraints ordering: RESOLVED
2. SecondWindWallAtMostOne — fire() unconditional spawn: RESOLVED
3. BoltCountReasonable — entropy_engine_stress threshold: RESOLVED (loosened via commit 53596f5)
4. Phase 3 FixedUpdate scheduling cycle: RESOLVED (2026-03-28)

## Coverage

103 scenario RON files total, 103 scenarios ran. No missing self-tests.

Stress suites passed:
- attraction_cell_chaos: 32/32
- breaker_oob_stress: 32/32
- cascade_shockwave_stress: 32/32
- chain_lightning_arc_lifecycle: 16/16
- chain_lightning_chaos: 16/16
- dense_stress: 32/32
- entropy_engine_stress: 32/32
- explode_chaos: 32/32
- gravity_well_chaos: 16/16
- phantom_bolt_stress: 16/16
- prism_scatter_stress: 32/32
- pulse_accumulation_stress: 32/32
- shield_cell_charge_depletion: 16/16
- spawn_bolts_stress: 16/16
- supernova_chain_stress: 16/16
- surge_speed_stress: 32/32
- tether_chain_bolt_stress: 16/16

## Note on output file reading

When reading background task output for `cargo scenario -- --all`, always read from the
*end* of the output file (use `tail`), not the middle. The stress suite summary lines
appear last and the file may still be growing when an earlier `cat` is captured.
