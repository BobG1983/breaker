---
name: Effect System Coverage Map
description: Which effects/triggers have scenario coverage and which are completely untested — updated develop branch full audit
type: project
---

## Effects with Scenario Coverage (develop branch state)

| Effect | Scenario(s) | Quality |
|--------|-------------|---------|
| SpeedBoost | surge_speed_stress, impacted_wall_speed, overclock_until_speed, initial_effects_bolt, passive_chips_chaos, early_late_bump_effects, node_start_speed_boost | Good — multiple triggers, stress, Until reversal |
| DamageBoost | passive_chips_chaos, damage_boost_until_reversal | Good — reversal path covered |
| Piercing | passive_chips_chaos | Minimal — no scenario verifying cell pass-through count |
| SizeBoost | passive_chips_chaos | Minimal — applied but no size invariant, no breaker SizeBoost scenario |
| Shockwave | surge_overclock, cascade_shockwave_stress, supernova_chain_stress, entropy_engine_stress, flux_random_chaos | Good |
| SpawnBolts | spawn_bolts_stress, supernova_chain_stress, entropy_engine_stress | Good (inherit:true path only in spawn_bolts_stress) |
| ChainBolt | tether_chain_bolt_stress | Good |
| EntropyEngine | entropy_engine_stress | Good |
| RandomEffect | flux_random_chaos | Good |
| SecondWind | bolt_lost_second_wind, second_wind_single_use | Good |
| Attraction | attraction_cell_chaos | Good |
| SpawnPhantom | phantom_bolt_stress | Good |
| Shield (Breaker) | shield_bolt_loss_prevention | Adequate for bolt-loss path |
| Shield (Cell) | shield_cell_charge_depletion | Good — stress 16 runs |
| GravityWell | gravity_well_chaos | Good |
| Pulse | pulse_accumulation_stress | Good |
| Explode | explode_chaos | Good |
| PiercingBeam | piercing_beam_stress | Good |
| TetherBeam | tether_beam_stress | Good |
| BumpForce | bump_force_stress | Good |
| RampingDamage | ramping_damage_reset | Good |
| QuickStop | quick_stop_dash_edges | Good |
| ChainLightning | chain_lightning_chaos + chain_lightning_arc_lifecycle | Good — ChainArcCountReasonable in both |
| TimePenalty | timer_threshold_penalty (via initial_effects) | MINIMAL — exercises timer subtraction path only; no scenario for the Chrono breaker's built-in TimePenalty on bolt loss |
| LoseLife | ZERO scenarios using LoseLife via initial_effects | NONE — aegis_lives_exhaustion uses organic bolt loss, not LoseLife effect directly |

## Effects with NO Scenario Coverage

- **LoseLife via initial_effects** — never exercised as an injected effect; only the built-in Aegis breaker definition triggers it organically. No scenario verifies LoseLife fires correctly when used as a chip effect.

## Triggers with NO Scenario Coverage (any initial_effects block)

- **NoBump** — no scenario ever uses `trigger: NoBump`
- **BumpWhiff** — no scenario ever uses `trigger: BumpWhiff` (despite Whiplash chip using it)
- **Death** — no scenario ever uses `trigger: Death`
- **Died** — no scenario ever uses `trigger: Died`
- **NodeEnd** — no scenario ever uses `trigger: NodeEnd`
- **Impact(Breaker)** — no scenario exercises Impact(Breaker) from any collision
- **Impact(Bolt)** — no scenario exercises Impact(Bolt) (emitted on every BoltImpact* message)
- **Impacted(Bolt)** on breaker — no scenario exercises this (BoltImpactBreaker → Impacted(Bolt) on breaker)
- **Impacted(Breaker)** — no scenario exercises this on any entity

## New Dispatch Paths Still Uncovered (since feature/source-chip-shield-absorption)

| New Dispatch Path | Scenario Coverage |
|-------------------|-------------------|
| BreakerImpactCell → Impacted(Cell) on breaker | NONE |
| BreakerImpactCell → Impacted(Breaker) on cell | NONE |
| BreakerImpactWall → Impacted(Wall) on breaker | NONE |
| BreakerImpactWall → Impacted(Breaker) on wall | NONE |

## Target Scope Coverage Gaps

- **AllBolts** target — only shield_cell_charge_depletion uses `AllCells`; **no scenario uses `On(target: AllBolts, ...)`** with a trigger. Parry chip fires Shockwave on AllBolts on PerfectBump — untested in scenarios.
- **AllCells** target beyond NodeStart — only NodeStart trigger used for AllCells.

## Chip-Level Coverage Gaps (chips with NO scenario exercising them)

- **Whiplash** — BumpWhiff trigger + Once wrapper + Shockwave + DamageBoost combination: zero coverage
- **Ricochet Protocol** — Until(Impacted(Cell)) removal from wall trigger: zero coverage
- **Feedback Loop** — 3-level nested trigger (PerfectBumped→Impacted(Cell)→CellDestroyed→Until): zero coverage
- **Chain Reaction** — nested CellDestroyed→CellDestroyed→SpawnBolts: only tested by supernova_chain_stress indirectly
- **Deadline** — NodeTimerThreshold(0.25) only; timer_threshold_penalty uses 0.5 threshold — distinct timing path
- **Tempo** — Until(BumpWhiff) removal path; zero coverage (BumpWhiff trigger never used in scenarios)
- **Parry** — AllBolts target + PerfectBump + Shield on Breaker simultaneous: zero coverage
- **Desperation / Last Stand** — BoltLost → SpeedBoost on Breaker (not bolt): only tested organically via aegis; no initial_effects scenario

## source_chip Attribution Coverage Gap (UNCHANGED)

- **DamageCell.source_chip propagation end-to-end** — unit tests verify each link. But NO scenario verifies the complete chain: chip-attributed bolt hits cell → DamageCell carries chip name → track_evolution_damage accumulates it.

## Invariant Gaps (develop branch)

Properties with no invariant checker:
- GravityWell entity count vs max cap: no invariant (gravity_well_chaos does not verify cap enforcement)
- RampingDamage accumulated bonus: NoNaN partially covers but no monotonicity invariant
- Cell ShieldActive charges never go negative: ShieldChargesConsistent covers zero, but negative unchecked
- DamageCell.source_chip correctness end-to-end: no invariant
- SizeBoost: no invariant validates breaker/bolt size stays within plausible bounds
- Bolt lifespan bolts are fully cleaned up: covered by NoEntityLeaks but no dedicated check
- AllBolts/AllCells effect targeting correctness: no invariant verifies all bolts/cells received an effect that was targeted at all of them

**How to apply:** When writing scenarios for new dispatch paths or chip types, flag missing invariants for BumpWhiff, NodeEnd, Death, Died triggers as HIGH priority.
