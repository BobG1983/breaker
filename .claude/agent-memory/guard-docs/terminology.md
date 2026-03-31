---
name: terminology
description: Confirmed term mappings and glossary gaps for the brickbreaker roguelite (updated for stat-effects phase)
type: project
---

## Confirmed Correct Terminology in Code

- `BoltImpactCell`, `BoltImpactWall`, `BoltImpactBreaker` — correct collision message names (not BoltHit*)
- `BreakerImpactCell`, `BreakerImpactWall` — breaker collision messages
- `CellImpactWall` — cell collision message
- `DamageCell.source_chip` — attribution field (not `source_bolt`)
- `EffectKind::SecondWind` — unit variant (no fields)
- `EffectKind::EntropyEngine { max_effects, pool }` — field is `max_effects`, NOT `threshold`
- `BoundEffects` — permanent chains on entities (not `ActiveEffects` or `ArmedEffects`)
- `StagedEffects` — one-shot chains on entities (not `EffectChains`)
- `EffectCommandsExt` — Commands extension for firing/reversing effects
- `SpawnBolts` — correct effect name (not `MultiBolt`)

## Active/Effective Component Pairs (stat-effects phase)

- `ActiveDamageBoosts` / `EffectiveDamageMultiplier` — damage stat (multiplicative)
- `ActiveSpeedBoosts` / `EffectiveSpeedMultiplier` — speed stat (multiplicative)
- `ActivePiercings` / `EffectivePiercing` — piercing stat (additive/sum)
- `ActiveSizeBoosts` / `EffectiveSizeMultiplier` — size stat (multiplicative)
- `ActiveBumpForces` / `EffectiveBumpForce` — bump force stat (multiplicative)
- `ActiveQuickStops` / `EffectiveQuickStop` — deceleration multiplier (multiplicative)
- `PiercingRemaining` — bolt gameplay state (bolt domain), NOT an effect stat

## New System Set Variants (stat-effects phase)

- `EffectSystems::Recalculate` — recalculate systems computing Active* → Effective* (runs after Bridge)
- `BoltSystems::CellCollision` — tags `bolt_cell_collision` (added when stat-effects needed ordering anchor)
- `BreakerSystems::UpdateState` — tags `update_breaker_state`

## Wave 3 New Types (feature/scenario-coverage)

- `TetherChainBeam` — marker component on chain-mode beam entities (distinguishes from standard TetherBeamComponent)
- `TetherChainActive` — resource inserted when TetherBeam chain mode is active; stores damage_mult, effective_damage_multiplier, source_chip, last_bolt_count
- `maintain_tether_chain` — FixedUpdate system; rebuilds chain beams when bolt count changes; gated on resource_exists::<TetherChainActive>; runs before tick_tether_beam
- `FlashStepActive` — marker component inserted by FlashStep::fire() on breaker entity
- `AnchorActive { bump_force_multiplier, perfect_window_multiplier, plant_delay }` — config component for Anchor effect
- `AnchorTimer(f32)` — countdown timer; inserted when breaker stops moving
- `AnchorPlanted` — marker inserted when AnchorTimer reaches zero
- `CircuitBreakerCounter { remaining, config }` — counter component for CircuitBreaker effect
- `CircuitBreakerConfig { bumps_required, spawn_count, inherit, shockwave_range, shockwave_speed }` — config struct in circuit_breaker module
- `LastImpact { position: Vec2, side: ImpactSide }` — component on bolts (written by collision systems); read by MirrorProtocol::fire()
- `ImpactSide` — enum (Top, Bottom, Left, Right); used by MirrorProtocol

## ChainBolt Runtime Components (runtime-effects phase)

- `ChainBoltMarker(Entity)` — on chain bolt entity, pointing to its anchor entity
- `ChainBoltAnchor` — on anchor entity (marker, inserted by fire(), removed by reverse())
- `ChainBoltConstraint(Entity)` — on chain bolt entity, pointing to the DistanceConstraint entity
- `ActiveAttractions(Vec<AttractionEntry>)` — tracks active attraction entries (attraction effect)
- `AttractionEntry { attraction_type, force, active }` — individual attraction tracking struct

## Explode Runtime Pattern

- `ExplodeRequest { range, damage_mult }` — deferred request entity spawned by fire(). Consumed (despawned) by `process_explode_requests` in the same or next tick. Position stored in Transform.
- SpawnChainBolt and SpawnAdditionalBolt messages are REMOVED — chain_bolt::fire() and spawn_bolts::fire() spawn directly via &mut World.

## Source Chip Attribution (source-chip-shield-absorption phase)

- `source_chip: &str` — parameter on all `fire()`/`reverse()` free functions; passed through from `EffectCommandsExt`
- `EffectSourceChip(Option<String>)` — component on spawned AoE/spawn entities (shockwave, pulse, explode request, chain lightning chain, piercing beam request, tether beam). Carries attribution from dispatch to damage tick.
- `chip_attribution(s: &str) -> Option<String>` — empty → None, non-empty → Some. Helper in core/types.rs.
- `BoltSystems::WallCollision` — system set tagging `bolt_wall_collision`; runs after CellCollision

## ShieldActive Variants

- `ShieldActive { charges: u32 }` — only field. No `remaining`, no `owner`, no duration fields.
- Charge decrement: handled in `bolt_lost` (breaker shield) and `handle_cell_hit` (cell shield) — NOT in a dedicated shield tick system.

## Intentional Shorthand in Docs (not drift)

- chip-catalog.md trigger notation: `When(PerfectBumped)`, `When(OnBump)`, `When(OnBoltLost)` — authoring shorthand matching Trigger enum variants loosely. These are design docs, not RON syntax specs.
