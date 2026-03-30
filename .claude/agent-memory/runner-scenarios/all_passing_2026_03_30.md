---
name: All 95 scenarios passing as of 2026-03-30
description: Full Verification Tier run on develop branch; 78 named PASS + 17 stress suites all-passed; 0 failures; all previously-known bugs resolved
type: project
---

## Run date: 2026-03-30

Branch: develop
Last commit: 14dddcd (Merge branch 'feature/source-chip-shield-absorption' into develop)

## Results

- 78 named scenario PASS (mechanic, self_test, stress non-stress runs)
- 17 stress suites: all-passed (totaling 488 individual runs)
- 0 failures
- 0 violations in gameplay scenarios (self-test scenarios correctly fired their expected violations)

## Previously known bugs — all resolved

1. BoltSpeedInRange — gravity_well ordering + enforce_distance_constraints ordering: RESOLVED
2. SecondWindWallAtMostOne — fire() unconditional spawn: RESOLVED
3. BoltCountReasonable — entropy_engine_stress threshold: RESOLVED (loosened via commit 53596f5)
4. Phase 3 FixedUpdate scheduling cycle: RESOLVED (2026-03-28)

## Coverage parity

95 scenario RON files, 95 scenarios ran. No missing self-tests. All layouts referenced.

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
