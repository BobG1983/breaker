---
name: Effect System Coverage Map
description: Which effects have scenario coverage and which are completely untested — updated after feature/runtime-effects branch audit
type: project
---

## Effects with Scenario Coverage (feature/runtime-effects state)

| Effect | Scenario(s) | Quality |
|--------|-------------|---------|
| SpeedBoost | surge_speed_stress, impacted_wall_speed, overclock_until_speed, initial_effects_bolt, passive_chips_chaos | Good — multiple triggers, stress, Until reversal |
| DamageBoost | passive_chips_chaos, damage_boost_until_reversal | Good — reversal path now covered |
| Piercing | passive_chips_chaos | Minimal — no scenario verifying cell pass-through count |
| SizeBoost | passive_chips_chaos | Minimal — applied but no size invariant checks it |
| Shockwave | surge_overclock, cascade_shockwave_stress, supernova_chain_stress, entropy_engine_stress, flux_random_chaos | Good |
| SpawnBolts | spawn_bolts_stress, supernova_chain_stress, entropy_engine_stress | Good |
| ChainBolt | tether_chain_bolt_stress | Good |
| EntropyEngine | entropy_engine_stress | Good |
| RandomEffect | flux_random_chaos | Good |
| SecondWind | bolt_lost_second_wind, second_wind_single_use | Good — now covered |
| Attraction | attraction_cell_chaos | Good — now covered |
| SpawnPhantom | phantom_bolt_stress | Good — now covered |
| Shield (Breaker) | shield_bolt_loss_prevention | Adequate for bolt-loss path; MISSING: cell ShieldActive charge decrement |
| GravityWell | gravity_well_chaos | Good — now covered |
| Pulse | pulse_accumulation_stress | Good — now covered |
| Explode | explode_chaos | Good — now covered |
| PiercingBeam | piercing_beam_stress | Good — now covered |
| TetherBeam | tether_beam_stress | Good — now covered |
| BumpForce | bump_force_stress | Good — now covered |
| RampingDamage | ramping_damage_reset | Good — now covered |
| QuickStop | quick_stop_dash_edges | Good — now covered |
| ChainLightning | chain_lightning_chaos | EXISTS but stale — scenario exercises the old instant batch model. New tick-based arc-traveling system (ChainLightningChain + tick_chain_lightning) is NOT verified by any scenario |

## Effects with NO Scenario Coverage

- **TimePenalty** — zero scenarios (chrono scenarios don't exercise it via initial_effects)
- **LoseLife** — zero scenarios (aegis_lives_exhaustion uses organic bolt loss, not LoseLife effect)

## Cell-Level Shield Coverage Gap (NEW — feature/runtime-effects)

- **ShieldActive on cells** — shield_bolt_loss_prevention only covers Breaker-level ShieldActive
  (bolt-loss absorption via bolt_lost system). The NEW cell-level ShieldActive charge-decrement
  behavior (handle_cell_hit absorbs DamageCell and decrements charges) has NO scenario coverage.
  Unit tests exist (shield_tests.rs — 12 behaviors) but no scenario verifies this under load.

## source_chip Attribution Coverage Gap (NEW — feature/runtime-effects)

- **DamageCell.source_chip propagation end-to-end** — unit tests verify:
  - bolt_cell_collision: SpawnedByEvolution → DamageCell.source_chip (attribution.rs)
  - ChainLightning fire() and tick_chain_lightning: EffectSourceChip → DamageCell.source_chip (fire_tests.rs, tick_tests.rs)
  - track_evolution_damage: accumulates per-chip damage (track_evolution_damage.rs)
  But NO scenario verifies the complete chain: chip-attributed bolt hits cell → DamageCell
  carries chip name → track_evolution_damage accumulates it. Integration path untested.

## Triggers with NO Scenario Coverage

- NoBump — never used in any initial_effects block
- BumpWhiff — never used in any initial_effects block
- Death / Died — never used in any initial_effects block
- NodeStart / NodeEnd — used (node_start_speed_boost, shield_bolt_loss_prevention) — now covered
- NodeTimerThreshold — timer_threshold_penalty now covers this
- BoltLost — godmode_breaker (self-test), bolt_lost_second_wind — now covered
- EarlyBump / EarlyBumped / LateBump / LateBumped — only in self-test perfect_input_* scenarios; early_late_bump_effects now covers EarlyBump/LateBump

## Invariant Gaps

Properties with no invariant checker:
- Active bolt count > configured max_bolt_count: partially covered by BoltCountReasonable but not per-effect
- SecondWind wall entity count (should be 0 or 1): SecondWindWallAtMostOne invariant EXISTS — self-test exists too
- GravityWell entity count vs max cap: no invariant
- RampingDamage accumulated bonus is NaN-free and non-negative: NoNaN partially covers this
- Cell ShieldActive charges never go negative: ShieldChargesConsistent covers zero, but negative is not checked
- ChainLightningChain entity count not unbounded: no invariant (arc entity leak under rapid triggering)
- DamageCell.source_chip correctness end-to-end: no invariant

**How to apply:** When writing scenarios for these effects, flag missing invariants as HIGH priority.
