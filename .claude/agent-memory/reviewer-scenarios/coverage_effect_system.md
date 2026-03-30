---
name: Effect System Coverage Map
description: Which effects/triggers have scenario coverage and which are completely untested — updated post-unit-test-58 branch audit
type: project
---

## Effects with Scenario Coverage (develop branch state, post unit-test branch)

| Effect | Scenario(s) | Quality |
|--------|-------------|---------|
| SpeedBoost | surge_speed_stress, impacted_wall_speed, overclock_until_speed, initial_effects_bolt, passive_chips_chaos, early_late_bump_effects, node_start_speed_boost, breaker_impact_trigger_chaos, node_end_speed_purge | Good — multiple triggers, stress, Until reversal, NodeEnd purge now covered |
| DamageBoost | passive_chips_chaos, damage_boost_until_reversal, whiplash_whiff_chaos | Good — reversal path covered, BumpWhiff trigger now exercised |
| Piercing | passive_chips_chaos | Minimal — no scenario verifying cell pass-through count |
| SizeBoost | passive_chips_chaos | Minimal — applied but no size invariant, no breaker SizeBoost scenario |
| Shockwave | surge_overclock, cascade_shockwave_stress, supernova_chain_stress, entropy_engine_stress, flux_random_chaos, whiplash_whiff_chaos | Good |
| SpawnBolts | spawn_bolts_stress, supernova_chain_stress, entropy_engine_stress, supernova_active_play | Good (inherit:true path only in spawn_bolts_stress) |
| ChainBolt | tether_chain_bolt_stress | Good |
| EntropyEngine | entropy_engine_stress | Good |
| RandomEffect | flux_random_chaos | Good |
| SecondWind | bolt_lost_second_wind, second_wind_single_use | Good |
| Attraction | attraction_cell_chaos | Good |
| SpawnPhantom | phantom_bolt_stress | Good |
| Shield (Breaker) | shield_bolt_loss_prevention | Adequate for bolt-loss path |
| Shield (Cell) | shield_cell_charge_depletion | Good — stress 16 runs |
| GravityWell | gravity_well_chaos | Good — unit tests now cover Position2D/Transform correctness + cap enforcement |
| Pulse | pulse_accumulation_stress | Good |
| Explode | explode_chaos | Good |
| PiercingBeam | piercing_beam_stress | Good |
| TetherBeam | tether_beam_stress | Good |
| BumpForce | bump_force_stress | Good |
| RampingDamage | ramping_damage_reset | Good |
| QuickStop | quick_stop_dash_edges | Good |
| ChainLightning | chain_lightning_chaos + chain_lightning_arc_lifecycle, voltchain_cell_chain | Good — ChainArcCountReasonable in all |
| TimePenalty | timer_threshold_penalty (via initial_effects) | MINIMAL — exercises timer subtraction path only; no scenario for the Chrono breaker's built-in TimePenalty on bolt loss |
| LoseLife | ZERO scenarios using LoseLife via initial_effects | NONE — aegis_lives_exhaustion uses organic bolt loss, not LoseLife effect directly |
| Once wrapper | once_damage_single_fire | NEW — Once re-arm bug exercised (Impacted(Cell) trigger) |

## Effects with NO Scenario Coverage

- **LoseLife via initial_effects** — never exercised as an injected effect; only the built-in Aegis breaker definition triggers it organically. No scenario verifies LoseLife fires correctly when used as a chip effect.

## Triggers with NO Scenario Coverage (any initial_effects block)

- **NoBump** — no scenario ever uses `trigger: NoBump`
- **Death** now covered via cell_death_speed_burst (Death trigger, SpeedBoost). Previously ZERO.
- **BumpWhiff** now covered via whiplash_whiff_chaos (BumpWhiff trigger + Once + Shockwave). Previously ZERO.
- **Died** — no scenario ever uses `trigger: Died`
- **NodeEnd** now partially covered via node_end_speed_purge (NodeStart/NodeEnd cycle). Previously ZERO.
- **Impact(Breaker)** now covered via breaker_impact_trigger_chaos. Previously ZERO.
- **Impact(Bolt)** — no scenario exercises Impact(Bolt) (emitted on every BoltImpactCell message)
- **Impacted(Bolt)** on breaker — no scenario exercises this (BoltImpactBreaker → Impacted(Bolt) on breaker)
- **Impacted(Breaker)** — no scenario exercises this on any entity

## New Dispatch Paths — Coverage Status After This Branch

| New Dispatch Path | Scenario Coverage |
|-------------------|-------------------|
| BreakerImpactCell → Impact(Cell) globally | NONE — unit tests only |
| BreakerImpactCell → Impacted(Cell) on breaker | NONE — unit tests only |
| BreakerImpactWall → Impact(Wall) globally | NONE — unit tests only |
| BreakerImpactWall → Impacted(Wall) on breaker | NONE — unit tests only |
| CellImpactWall → Impact(Wall) globally | NONE — unit tests only |
| CellImpactWall → Impact(Cell) globally | NONE — unit tests only |
| BoltImpactBreaker → Impact(Breaker) globally | COVERED — breaker_impact_trigger_chaos |
| BoltImpactWall → Impact(Wall) globally | MINIMAL — impacted_wall_speed |

## Target Scope Coverage Gaps

- **AllBolts** target — only shield_cell_charge_depletion uses `AllCells`; **no scenario uses `On(target: AllBolts, ...)`** with a trigger AND an invariant that proves all bolts received the effect. (breaker_impact_trigger_chaos uses AllBolts but only BoltSpeedInRange, not a count check.)
- **AllCells** target beyond NodeStart — only NodeStart trigger used for AllCells.

## Chip-Level Coverage Gaps (chips with NO scenario exercising them)

- **Whiplash** — BumpWhiff trigger + Once wrapper + Shockwave + DamageBoost combination: now covered by whiplash_whiff_chaos
- **Ricochet Protocol** — Until(Impacted(Cell)) removal from wall trigger: zero coverage
- **Feedback Loop** — 3-level nested trigger (PerfectBumped→Impacted(Cell)→CellDestroyed→Until): zero coverage
- **Chain Reaction** — nested CellDestroyed→CellDestroyed→SpawnBolts: only tested by supernova_chain_stress indirectly
- **Deadline** — NodeTimerThreshold(0.25) only; timer_threshold_penalty uses 0.5 threshold — distinct timing path
- **Tempo** — Until(BumpWhiff) removal path; BumpWhiff now exercised by whiplash_whiff_chaos, but Tempo specifically is not
- **Parry** — AllBolts target + PerfectBump + Shield on Breaker simultaneous: zero coverage
- **Desperation / Last Stand** — BoltLost → SpeedBoost on Breaker (not bolt): only tested organically via aegis; no initial_effects scenario
- **Dead Man's Hand** evolution — now covered by dead_mans_hand_bolt_loss

## source_chip Attribution Coverage Gap (UNCHANGED)

- **DamageCell.source_chip propagation end-to-end** — unit tests verify each link. But NO scenario verifies the complete chain: chip-attributed bolt hits cell → DamageCell carries chip name → track_evolution_damage accumulates it.

## Collision System Coverage (new on this branch — unit tests only)

- **BreakerImpactCell/Wall dispatch** — 4 new unit tests (impact + impacted dirs, breaker_collision_tests.rs). No scenario coverage.
- **CellImpactWall dispatch** — 2 new unit tests. No scenario coverage.
- **Quadtree layer filter (query_aabb_filtered / query_circle_filtered)** — 11 new unit tests in layer_filter_tests.rs. No scenario exercises filtered queries directly.
- **CCD reads Aabb2D not legacy dimensions** — 4 new unit tests in aabb_collision.rs. No scenario for "bolt misses cell due to custom Aabb2D being smaller than visual dimensions."
- **EntityScale-aware bolt collision** — 2 new unit tests (scaled_bolt_effective_radius). No scenario for scaled bolt collisions specifically.
- **GravityWell Position2D correctness** — 14 new unit tests. Existing gravity_well_chaos scenario does NOT verify Position2D vs Transform correctness — it only verifies entity lifecycle and speed bounds.

## Invariant Gaps (develop branch)

Properties with no invariant checker:
- GravityWell entity count vs max cap: no invariant (gravity_well_chaos does not verify cap enforcement — unit tests now verify this in isolation)
- RampingDamage accumulated bonus: NoNaN partially covers but no monotonicity invariant
- Cell ShieldActive charges never go negative: ShieldChargesConsistent covers zero, but negative unchecked
- DamageCell.source_chip correctness end-to-end: no invariant
- SizeBoost: no invariant validates breaker/bolt size stays within plausible bounds
- Bolt lifespan bolts are fully cleaned up: covered by NoEntityLeaks but no dedicated check
- AllBolts/AllCells effect targeting correctness: no invariant verifies all bolts/cells received an effect that was targeted at all of them
- Quadtree layer filter correctness at runtime: no invariant verifies that query_aabb_filtered returns only entities on the correct collision layers during gameplay

**How to apply:** When writing scenarios for new dispatch paths or chip types, flag missing invariants for BumpWhiff, NodeEnd, Death, Died triggers as HIGH priority. BreakerImpactCell/Wall dispatch paths (Impact(Cell)/Impacted(Cell) on breaker) remain scenario-uncovered despite new unit tests.
