---
name: All 103 scenarios passing as of 2026-03-30 (post-new-scenarios)
description: Full run on develop branch after adding 8 new scenarios; 86 named PASS + 15 stress suites all-passed; 0 failures; all previously-known bugs remain resolved
type: project
---

## Run date: 2026-03-30

Branch: develop
Last commit: 14dddcd (Merge branch 'feature/source-chip-shield-absorption' into develop)

## Results

- 86 named scenario PASS (chaos, mechanic, self_test, and non-stress stress scenarios)
- 17 stress suites: all-passed
- 0 failures
- 0 violations in gameplay scenarios (self-test scenarios correctly fired their expected violations)

## New scenarios added (all PASS)

chaos/whiplash_whiff_chaos
chaos/node_end_speed_purge
chaos/cell_death_speed_burst
chaos/breaker_impact_trigger_chaos
mechanic/once_damage_single_fire
mechanic/supernova_active_play
mechanic/dead_mans_hand_bolt_loss
mechanic/voltchain_cell_chain

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
