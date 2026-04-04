---
name: Effect System Coverage Map
description: Which effects/triggers have scenario coverage and which are completely untested — updated Wave 3 (tether chain mode, spawn_bolts inherit fix, maintain_tether_chain)
type: project
---

## Wave 3 New Mechanics — Coverage Status (feature/scenario-coverage branch)

| Mechanic | Unit tests | Scenario coverage | Quality |
|----------|-----------|-------------------|---------|
| TetherBeam chain mode (`chain: true`) | maintain_chain_tests.rs — thorough | NONE — tether_beam_stress only exercises standard mode (`chain: false`, `damage_mult: 1.5`) | MISSING |
| SpawnBolts inherit fix | fire_inherit.rs — thorough | spawn_bolts_stress uses `inherit: true` but only tests BoltCountReasonable/NoEntityLeaks/BoltInBounds/NoNaN — no invariant verifies inherited BoundEffects actually fire on spawned bolts | WEAK |
| maintain_tether_chain | maintain_chain_tests.rs — thorough (bolt lost, bolt spawned, no-op, resource absent, rebuild props, all-dead, repair-from-zero) | NONE — no scenario exercises TetherChainActive dynamic rebuild under chaos | MISSING |

## Effects with Scenario Coverage (develop branch state, post 3-invariant / 8-scenario branch)

| Effect | Scenario(s) | Quality |
|--------|-------------|---------|
| SpeedBoost | surge_speed_stress, impacted_wall_speed, overclock_until_speed, initial_effects_bolt, passive_chips_chaos, early_late_bump_effects, node_start_speed_boost, breaker_impact_trigger_chaos, node_end_speed_purge, cell_death_speed_burst | Good — Death trigger, NodeEnd purge, reversal all covered |
| DamageBoost | passive_chips_chaos, damage_boost_until_reversal, whiplash_whiff_chaos, once_damage_single_fire | Good |
| Piercing | passive_chips_chaos | Minimal — no scenario verifying cell pass-through count |
| SizeBoost | passive_chips_chaos, entity_scale_collision_chaos | Minimal — passive_chips_chaos exercises SizeBoost but no dedicated SizeBoost invariant exists (SizeBoostInRange was REMOVED with Effective* cache removal); entity_scale_collision_chaos exercises Aabb2D integrity |
| Shockwave | surge_overclock, cascade_shockwave_stress, supernova_chain_stress, entropy_engine_stress, flux_random_chaos, whiplash_whiff_chaos | Good |
| SpawnBolts | spawn_bolts_stress, supernova_chain_stress, entropy_engine_stress, supernova_active_play | Good |
| ChainBolt | tether_chain_bolt_stress | Good |
| EntropyEngine | entropy_engine_stress | Good |
| RandomEffect | flux_random_chaos | Good |
| SecondWind | bolt_lost_second_wind, second_wind_single_use | Good |
| Attraction | attraction_cell_chaos | Good |
| SpawnPhantom | phantom_bolt_stress | Good |
| Shield | shield_wall_reflection (mechanic), shield_wall_at_most_one (self-test) | Adequate — reflection and timer lifecycle covered; no chaos scenario |
| GravityWell | gravity_well_chaos, gravity_well_stress | Good — gravity_well_stress uses GravityWellCountReasonable invariant |
| Pulse | pulse_accumulation_stress | Good |
| Explode | explode_chaos | Good |
| PiercingBeam | piercing_beam_stress | Good |
| TetherBeam (standard mode) | tether_beam_stress | Good — standard mode only |
| TetherBeam (chain mode) | NONE | MISSING — chain mode never exercised by any scenario |
| BumpForce | bump_force_stress | Good |
| RampingDamage | ramping_damage_reset | Weak — still only NoNaN+BoltInBounds; no monotonicity invariant |
| QuickStop | quick_stop_dash_edges | Good |
| ChainLightning | chain_lightning_chaos, chain_lightning_arc_lifecycle, voltchain_cell_chain | Good |
| TimePenalty | timer_threshold_penalty | MINIMAL — timer subtraction path only; no Chrono built-in TimePenalty-on-bolt-loss scenario |
| LoseLife | ZERO scenarios using LoseLife via initial_effects | NONE |
| Once wrapper | once_damage_single_fire | Good — Once re-arm bug exercised |

## Effects with NO Scenario Coverage

- **LoseLife via initial_effects** — never exercised as an injected effect.

## Triggers with NO Scenario Coverage

- **NoBump** — no scenario uses `trigger: NoBump`
- **Died** — no scenario uses `trigger: Died`
- **Impact(Bolt)** — no scenario exercises Impact(Bolt) (BoltImpactCell → Impact(Bolt) on bolt)
- **Impacted(Bolt) on breaker** — no scenario (BoltImpactBreaker → Impacted(Bolt) on breaker)
- **Impacted(Breaker)** — no scenario exercises this on any entity

## New Dispatch Paths — Coverage Status After This Branch

| New Dispatch Path | Scenario Coverage |
|-------------------|-------------------|
| BreakerImpactCell → Impact(Cell) globally | breaker_cell_impact_chaos — but NO InvariantKind verifies the effect fired, only crash guards |
| BreakerImpactCell → Impacted(Cell) on breaker | NONE — unit tests only |
| BreakerImpactWall → Impact(Wall) globally | breaker_wall_impact_chaos — crash guards only |
| BreakerImpactWall → Impacted(Wall) on breaker | NONE — unit tests only |
| CellImpactWall → Impact(Wall) globally | cell_wall_proximity — NONE invariant proves trigger fired |
| CellImpactWall → Impact(Cell) globally | NONE |
| BoltImpactBreaker → Impact(Breaker) globally | breaker_impact_trigger_chaos |
| BoltImpactWall → Impact(Wall) globally | impacted_wall_speed |

## New Invariants — Non-Self-Test Usage

| Invariant | Self-test | Non-self-test scenarios |
|-----------|-----------|------------------------|
| AabbMatchesEntityDimensions | aabb_matches_entity_dimensions | entity_scale_collision_chaos only |
| GravityWellCountReasonable | gravity_well_count_reasonable | gravity_well_stress |
| SizeBoostInRange | N/A — REMOVED with Effective* cache removal | REMOVED — do NOT reference this invariant |

## Adversarial Quality Issues with New Scenarios

- **breaker_cell_impact_chaos**: Only crash guards [NoNaN, BoltInBounds, NoEntityLeaks, BreakerInBounds]. No invariant proves BreakerImpactCell effects actually fired correctly. Missing AabbMatchesEntityDimensions.
- **breaker_wall_impact_chaos**: Only crash guards [NoNaN, BoltInBounds, BreakerInBounds, BoltSpeedAccurate]. No invariant proves BreakerImpactWall effects fired. Missing AabbMatchesEntityDimensions.
- **entity_scale_collision_chaos**: Correct use of AabbMatchesEntityDimensions. SizeBoostInRange was REMOVED — no dedicated SizeBoost value-range invariant exists. The scenario still exercises SizeBoost per-bump but there is no invariant verifying multiplier range.
- **gravity_well_stress**: Good — uses GravityWellCountReasonable with stress(32). Appropriate.
- **cell_wall_proximity**: Minimal scenario (no initial_effects, Scripted empty input). Only [NoNaN, NoEntityLeaks]. Does NOT verify any effect fired on CellImpactWall — no initial_effects with Impact(Wall) trigger to prove the dispatch path works.

## Invariant Gaps (this branch state)

Properties with no invariant checker:
- SizeBoost: SizeBoostInRange REMOVED (Effective* cache removal). No invariant checks multiplier stays within a plausible range (e.g., never exceeds 100x from runaway stacking)
- RampingDamage accumulated bonus: NoNaN partially covers but no monotonicity invariant
- Shield wall count invariant: ShieldWallAtMostOne checks count <= 1; no chaos scenario exercises sustained Shield firing
- DamageCell.source_chip correctness end-to-end: no invariant
- AllBolts/AllCells effect targeting correctness: no invariant verifies all entities received an effect
- Quadtree layer filter correctness at runtime: no invariant

**How to apply:** Flag new dispatch path scenarios (breaker_cell_impact_chaos, breaker_wall_impact_chaos) as testing "doesn't crash" not "works correctly." SizeBoostInRange no longer exists — do not flag its absence as a gap.
