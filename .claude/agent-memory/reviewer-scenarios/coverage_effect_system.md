---
name: Effect System Coverage Map
description: Which effects/triggers have scenario coverage and which are completely untested — updated after feature/source-chip-shield-absorption branch audit
type: project
---

## Effects with Scenario Coverage (feature/source-chip-shield-absorption state)

| Effect | Scenario(s) | Quality |
|--------|-------------|---------|
| SpeedBoost | surge_speed_stress, impacted_wall_speed, overclock_until_speed, initial_effects_bolt, passive_chips_chaos | Good — multiple triggers, stress, Until reversal |
| DamageBoost | passive_chips_chaos, damage_boost_until_reversal | Good — reversal path covered |
| Piercing | passive_chips_chaos | Minimal — no scenario verifying cell pass-through count |
| SizeBoost | passive_chips_chaos | Minimal — applied but no size invariant checks it |
| Shockwave | surge_overclock, cascade_shockwave_stress, supernova_chain_stress, entropy_engine_stress, flux_random_chaos | Good |
| SpawnBolts | spawn_bolts_stress, supernova_chain_stress, entropy_engine_stress | Good |
| ChainBolt | tether_chain_bolt_stress | Good |
| EntropyEngine | entropy_engine_stress | Good |
| RandomEffect | flux_random_chaos | Good |
| SecondWind | bolt_lost_second_wind, second_wind_single_use | Good |
| Attraction | attraction_cell_chaos | Good |
| SpawnPhantom | phantom_bolt_stress | Good |
| Shield (Breaker) | shield_bolt_loss_prevention | Adequate for bolt-loss path |
| Shield (Cell) | shield_cell_charge_depletion | NEW — added this branch. Covers charge decrement via Shockwave + Dense layout. Good. |
| GravityWell | gravity_well_chaos | Good |
| Pulse | pulse_accumulation_stress | Good |
| Explode | explode_chaos | Good |
| PiercingBeam | piercing_beam_stress | Good |
| TetherBeam | tether_beam_stress | Good |
| BumpForce | bump_force_stress | Good |
| RampingDamage | ramping_damage_reset | Good |
| QuickStop | quick_stop_dash_edges | Good |
| ChainLightning | chain_lightning_chaos + chain_lightning_arc_lifecycle | NEW arc model now covered. chain_lightning_chaos uses default arc_speed; arc_lifecycle uses slow arc_speed: 50.0 for mid-flight destruction. ChainArcCountReasonable invariant adds leak detection. |

## Effects with NO Scenario Coverage

- **TimePenalty** — zero scenarios (chrono scenarios don't exercise it via initial_effects)
- **LoseLife** — zero scenarios (aegis_lives_exhaustion uses organic bolt loss, not LoseLife effect)

## New Dispatch Paths from feature/source-chip-shield-absorption Branch

This branch added 6 new Impact trigger bridge systems (global) and 6 new Impacted trigger bridge systems (targeted):

| New Dispatch Path | Scenario Coverage |
|-------------------|-------------------|
| BreakerImpactCell → Impacted(Cell) on breaker | NONE — no scenario uses `On(target: Breaker, When: Impacted(Cell))` |
| BreakerImpactCell → Impacted(Breaker) on cell | NONE — no scenario uses `On(target: Cell, When: Impacted(Breaker))` |
| BreakerImpactWall → Impacted(Wall) on breaker | NONE — no scenario uses `On(target: Breaker, When: Impacted(Wall))` |
| BreakerImpactWall → Impacted(Breaker) on wall | NONE — no scenario uses `On(target: Wall, When: Impacted(Breaker))` |
| CellImpactWall → Impacted(Wall) on cell | NONE — CellImpactWall message is "for future moving-cell mechanics" |
| CellImpactWall → Impacted(Cell) on wall | NONE — same |
| BoltImpactBreaker → Impacted(Bolt) on breaker | quick_stop_dash_edges uses On(Breaker, Bumped) not Impacted(Bolt) — NONE for Impacted(Bolt) on breaker specifically |
| Impact(Breaker) global | NONE — no scenario exercises Impact(Breaker) from any collision |
| Impact(Cell) global from BreakerImpactCell | NONE |

Unit tests exist for all bridge systems (impacted.rs tests module, impact.rs tests module) but NO scenario verifies these paths under load with real game entities.

## Cell-Level Shield Coverage

- **ShieldActive on cells** — now covered by shield_cell_charge_depletion (added this branch).
  Stress: 16 runs, Dense layout, 8000 frames. Covers charge decrement and ShieldChargesConsistent.

## source_chip Attribution Coverage Gap (UNCHANGED from prior branch)

- **DamageCell.source_chip propagation end-to-end** — unit tests verify each link.
  But NO scenario verifies the complete chain: chip-attributed bolt hits cell → DamageCell
  carries chip name → track_evolution_damage accumulates it. Integration path untested.

## Triggers with NO Scenario Coverage

- NoBump — never used in any initial_effects block
- BumpWhiff — never used in any initial_effects block
- Death / Died — never used in any initial_effects block
- Impacted(Breaker) — no scenario exercises this trigger firing on any entity

## Invariant Gaps (updated)

Properties with no invariant checker:
- Active bolt count > configured max_bolt_count: partially covered by BoltCountReasonable but not per-effect
- GravityWell entity count vs max cap: no invariant
- RampingDamage accumulated bonus is NaN-free and non-negative: NoNaN partially covers this
- Cell ShieldActive charges never go negative: ShieldChargesConsistent covers zero, but negative is not checked
- DamageCell.source_chip correctness end-to-end: no invariant
- ChainArcCountReasonable: NEW invariant added this branch — self-test exists (chain_arc_count_exceeded). chaos scenario is chain_lightning_chaos (uses ChainArcCountReasonable? NO — see below)

## ChainArcCountReasonable Invariant Status

- Self-test scenario: chain_arc_count_exceeded.scenario.ron — EXISTS, uses SpawnExtraChainArcs frame mutation. Good.
- Chaos/stress scenario: chain_lightning_chaos does NOT include ChainArcCountReasonable in its invariants list — it only has NoEntityLeaks, NoNaN, BoltInBounds, ShieldChargesConsistent.
- chain_lightning_arc_lifecycle does NOT include ChainArcCountReasonable either — only NoEntityLeaks, NoNaN, BoltInBounds.
- RESULT: The new invariant has a self-test but NO chaos/stress scenario enables it. This is a gap.

**How to apply:** When writing scenarios for new dispatch paths, flag missing invariants as HIGH priority.
