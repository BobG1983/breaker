---
name: Evolution Ecosystem Coverage Map
description: Scenario and invariant gaps introduced by the chip-evolution-ecosystem branch — 16 evolution RON files, 4 new effects (CircuitBreaker, MirrorProtocol, Anchor, FlashStep), ChipOfferExpected mechanic
type: project
---

## New mechanics on feature/chip-evolution-ecosystem

### New Effects (all have unit tests; coverage by scenario runner below)

| Effect | Scenario coverage | Quality |
|--------|-------------------|---------|
| CircuitBreaker (counter countdown + bolt spawn + shockwave) | NONE — no scenario exercises CircuitBreaker counter cycle | MISSING |
| MirrorProtocol (spawn mirrored bolt from LastImpact) | NONE — no scenario exercises MirrorProtocol bolt spawn | MISSING |
| Anchor (stationary plant delay → BumpForce + perfect window boost) | NONE — no scenario exercises Anchor planting or multiplier | MISSING |
| FlashStep (marker inserts on fire, dash system uses it) | NONE — no scenario exercises FlashStep dash teleport | MISSING |

### Evolution RON Files vs Scenario Coverage

| Evolution | Scenario | Quality |
|-----------|----------|---------|
| Supernova | evolution_supernova.scenario.ron | Weak — only verifies `expected_offerings: ["Supernova"]`; no frames of active play with supernova effects firing |
| Voltchain | NONE (voltchain_cell_chain.scenario.ron exists — exercises ChainLightning CellDestroyed, but NOT via the Voltchain evolution chip) | Gap: voltchain_cell_chain uses initial_effects, not chip_selections |
| Phantom Bolt | NONE — phantom_bolt_stress uses initial_effects SpawnPhantom, not the Phantom Bolt evolution | MISSING |
| Gravity Well (evolution) | NONE — gravity_well_chaos/stress use initial_effects, not the evolution chip | MISSING |
| Dead Man's Hand | dead_mans_hand_bolt_loss.scenario.ron | Adequate — uses chip_selections ["Dead Man's Hand"] |
| Second Wind (evolution) | bolt_lost_second_wind / second_wind_single_use — but these use initial_effects, not chip_selections | Gap |
| Entropy Engine (evolution) | entropy_engine_stress uses initial_effects, not chip_selections | Gap |
| Split Decision | NONE | MISSING |
| Nova Lance | NONE | MISSING |
| Circuit Breaker | NONE | MISSING |
| Mirror Protocol | NONE | MISSING |
| Arcwelder (TetherBeam chain:true) | tether_beam_stress — but uses initial_effects and standard mode. Chain mode still MISSING | MISSING |
| Anchor | NONE | MISSING |
| FlashStep | dash_edges exercises dash system; quick_stop_dash_edges exercises QuickStop. Neither exercises FlashStep | MISSING |
| Resonance Cascade | pulse_accumulation_stress uses initial_effects Pulse, not the Resonance Cascade evolution | Gap |
| Shock Chain | cascade_shockwave_stress uses initial_effects; no chip_selections Shock Chain | Gap |

### ChipCatalog Build + Recipe Eligibility Coverage

| Behavior | Coverage | Quality |
|----------|----------|---------|
| EvolutionTemplateRegistry seeded at boot | Unit tests (resources/tests/evolution_template_registry.rs) | Good |
| ChipCatalog.recipes populated from evolutions | Unit tests (build_chip_catalog tests) | Good |
| eligible_recipes returns correct evolutions for inventory | Unit tests (chip_catalog.rs) | Good |
| Evolutions excluded from regular offering pool | Unit tests | Good |
| Evolution offered at boss node when eligible | evolution_supernova verifies expected_offerings | Minimal — single evolution, single boss cycle |
| Evolution offered when multiple evolutions eligible | NONE | MISSING |
| No evolution offered when ingredients not met | NONE | MISSING |
| ChipOfferExpected fires on missing expected chip | Self-test: chip_offer_expected_self_test.scenario.ron | Good |

### Invariant Gaps for Evolution Mechanics

- No `EvolutionIngredientsSatisfied` invariant: if eligible_recipes wrongly includes non-eligible evolutions, no checker fires
- No `EvolutionNeverInRegularPool` invariant: evolutions excluded from pool via `chip.rarity == Rarity::Evolution` check — no runtime invariant verifies this
- No `CircuitBreakerCounterSane` invariant: CircuitBreakerCounter.remaining should be >0 and <=bumps_required; unchecked
- No `AnchorStateConsistent` invariant: AnchorTimer and AnchorPlanted mutual exclusivity unchecked
- No `MirrorProtocolBoltCount` invariant: MirrorProtocol spawns bolts that should obey BoltCountReasonable (covered generically), but no invariant verifies the mirrored bolt's LastImpact data is valid

## How to apply

- Flag all four new effects (CircuitBreaker, MirrorProtocol, Anchor, FlashStep) as HIGH gaps — unit tests exist but no chaos scenarios
- Flag evolution offering path (eligible chips shown at boss, ineligible chips not shown) as HIGH gap — evolution_supernova only tests the happy path
- Flag Arcwelder (chain mode TetherBeam) as HIGH gap — carried forward from prior audit, still unresolved
- Flag Split Decision, Nova Lance as HIGH gaps — new effects, no scenarios
- Flag Anchor, FlashStep as HIGH gaps — runtime systems (tick_anchor, breaker dash system) need scenario coverage
