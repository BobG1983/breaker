---
name: phase-log
description: Record of doc review sessions and what was updated (dates and scope)
type: project
---

## 2026-03-28 — feature/collision-cleanup review

**Branch:** feature/collision-cleanup (commit 35c10d1 effect system rewrite)

**Files reviewed:**
- `breaker-game/src/bolt/messages.rs`, `breaker/messages.rs`, `cells/messages.rs`
- `breaker-game/src/run/messages.rs`, `run/node/messages.rs`
- `breaker-game/src/ui/messages.rs`, `wall/messages.rs`
- `breaker-game/src/effect/core/types.rs`, `mod.rs`, `commands.rs`, `sets.rs`
- `breaker-game/src/effect/effects/mod.rs`, `speed_boost.rs`
- `breaker-game/src/effect/triggers/mod.rs`
- `breaker-game/src/game.rs`

**Docs updated:**
- `docs/architecture/messages.md` — 6 changes (collision message renames, new messages, DamageCell field, Observer Events section replaced)
- `docs/architecture/plugins.md` — 2 changes (cross-domain message names, full Effect Domain section rewrite)
- `docs/architecture/effects/core_types.md` — 2 changes (EffectKind enum corrected)
- `docs/architecture/effects/reversal.md` — 1 change (passive buffs and new effect types table)
- `docs/architecture/effects/node_types.md` — 1 change (SecondWind unit variant example)
- `docs/architecture/layout.md` — 1 change (effect domain file structure)

**Items confirmed no-drift:**
- `docs/design/chip-catalog.md` — TiltControl and MultiBolt were never present; SpawnBolts correct
- `docs/design/effects/ramping_damage.md` — `damage_per_trigger` matches code
- `docs/plan/index.md` — phase completion status accurate

## 2026-03-28 — feature/runtime-effects review

**Branch:** feature/runtime-effects (2 commits: 5edba37 parent, implementing 15+ effect runtime behaviors)

**Files reviewed:**
- `breaker-game/src/effect/effects/` — all 25 modules (attraction, chain_bolt, explode, pulse, second_wind, shockwave, spawn_phantom, and all prior stat effects)
- `breaker-game/src/bolt/messages.rs` — SpawnAdditionalBolt status
- `breaker-game/src/bolt/plugin.rs` — system registration
- `breaker-game/src/bolt/systems/bolt_lost/system.rs` — two-phase destruction, ShieldActive read
- `breaker-game/src/cells/queries.rs` — ShieldActive in DamageVisualQuery
- `breaker-game/src/cells/systems/handle_cell_hit/system.rs` — ShieldActive immunity

**Drifts found and fixed:**
- `docs/design/effects/explode.md` — removed "Not yet implemented" status line (explode is fully implemented)
- `docs/architecture/messages.md` — DamageCell sender list expanded (shockwave was only entry; explode, pulse, chain_lightning, piercing_beam, tether_beam added)
- `docs/architecture/messages.md` — removed SpawnChainBolt row (message type does not exist; chain_bolt::fire() spawns directly)
- `docs/architecture/messages.md` — moved SpawnAdditionalBolt from Active to new Registered section (registered but no producer or consumer)
- `docs/architecture/ordering.md` — removed spawn_additional_bolt and spawn_chain_bolt system entries (neither system exists)
- `docs/design/terminology/core.md` — corrected ChainBolt entry (was: SpawnChainBolt message, ChainHit effect; now: ChainBolt effect, direct world spawn, correct component names)
- `docs/plan/index.md` — added Runtime Effects entry to Current section (In Progress, feature/runtime-effects)

**Items confirmed no-drift:**
- `docs/architecture/effects/core_types.md` — EffectKind enum matches code, AttractionType correct
- `docs/design/effects/` — all design docs match implemented behavior (pulse interval hardcoded at 0.5s, not documented — intentional, internal detail)
- `docs/design/effects/attraction.md` — "nearest wins" and type deactivation/reactivation matches code exactly
- `BoltImpactWall` consumer entry ("effect") covers both bridge triggers AND runtime effect systems — no change needed

## 2026-03-29 — feature/source-chip-shield-absorption (second review session)

**Branch:** feature/source-chip-shield-absorption

**Files reviewed:**
- `breaker-game/src/effect/commands.rs` — push_bound_effects method, PushBoundEffects command
- `breaker-game/src/cells/components/types.rs` — CellEffectsDispatched marker component
- `breaker-game/src/cells/systems/dispatch_cell_effects.rs` — dispatch logic, CellDispatchQuery
- `breaker-game/src/cells/plugin.rs` — dispatch_cell_effects registration after NodeSystems::Spawn
- `breaker-game/src/wall/systems/dispatch_wall_effects.rs` — no-op stub
- `breaker-game/src/wall/plugin.rs` — (spawn_walls, dispatch_wall_effects).chain() registration
- `breaker-game/src/breaker/systems/dispatch_breaker_effects/system.rs` — dispatch logic
- `breaker-game/src/breaker/plugin.rs` — dispatch_breaker_effects registration
- `breaker-scenario-runner/src/types/mod.rs` — InvariantKind (22 variants), MutationKind (15 variants)
- `breaker-scenario-runner/src/invariants/checkers/check_chain_arc_count_reasonable.rs` — new checker
- `breaker-scenario-runner/src/invariants/checkers/mod.rs` — confirmed check_chain_arc_count_reasonable wired

**Drifts found and fixed:**
- `docs/architecture/messages.md` — fire_effect/reverse_effect missing source_chip param; push_bound_effects not listed
- `docs/architecture/plugins.md` — fire_effect/reverse_effect missing source_chip; push_bound_effects absent from Effect Dispatch section
- `docs/architecture/effects/commands.md` — EffectCommandsExt trait missing push_bound_effects; PushBoundEffects command not documented
- `docs/architecture/standards.md` — invariant list missing 6 variants (ChipOfferExpected, SecondWindWallAtMostOne, ShieldChargesConsistent, PulseRingAccumulation, EffectiveSpeedConsistent, ChainArcCountReasonable); count "16" changed to "22" (twice)
- `docs/design/terminology/scenarios.md` — MutationKind listed only 5 variants; expanded to all 15
- `docs/architecture/ordering.md` — OnEnter(GameState::Playing) chain missing dispatch_cell_effects, dispatch_wall_effects, dispatch_breaker_effects

**Items confirmed no-drift:**
- `lib.rs` pub mod visibility — cells and wall already listed in scenario runner exception
- game.rs plugin registration order — unchanged
- InvariantKind ALL slice — matches enum variants exactly

## 2026-03-29 — feature/runtime-effects source-chip-shield-absorption review (first session)

**Branch:** feature/runtime-effects (source_chip and shield absorption phase)

**Files reviewed:**
- `breaker-game/src/effect/core/types.rs` — EffectKind, EffectSourceChip, chip_attribution, fire/reverse method signatures
- `breaker-game/src/effect/commands.rs` — EffectCommandsExt signatures with source_chip
- `breaker-game/src/effect/effects/shield.rs` — Shield { stacks } only, fire/reverse/register signatures
- `breaker-game/src/effect/effects/chain_lightning/effect.rs` — arc_speed field, EffectSourceChip usage
- `breaker-game/src/bolt/sets.rs` — WallCollision variant present
- `breaker-game/src/bolt/plugin.rs` — WallCollision registration
- `breaker-game/src/bolt/systems/bolt_lost/system.rs` — ShieldActive cross-domain write
- `breaker-game/src/cells/systems/handle_cell_hit/system.rs` — ShieldActive cross-domain write
- `docs/design/effects/shield.md` — verified accurate
- `docs/design/effects/chain_lightning.md` — verified accurate (arc_speed not in parameters table — see needs-human note)

**Drifts found and fixed:**
- `docs/architecture/effects/core_types.md` — Attraction variant: tuple → named fields with max_force
- `docs/architecture/effects/core_types.md` — Shield variant: old {base_duration, duration_per_level, stacks} → {stacks} only
- `docs/architecture/effects/core_types.md` — ChainLightning variant: added arc_speed field (default 200.0)
- `docs/architecture/effects/core_types.md` — Pulse variant: added interval field (default 0.5)
- `docs/architecture/effects/core_types.md` — fire()/reverse() signatures: added source_chip: &str param
- `docs/architecture/effects/core_types.md` — method split: updated from "fire + fire_aoe_and_spawn" to "fire + fire_aoe_and_spawn + fire_utility_and_spawn" (3 fire methods), "reverse + reverse_aoe_and_spawn" (2 reverse methods)
- `docs/architecture/effects/core_types.md` — EffectSourceChip type + chip_attribution(): new section added
- `docs/architecture/effects/core_types.md` — Per-Effect Modules: added source_chip param to function signatures
- `docs/architecture/effects/commands.md` — fire_effect/reverse_effect signatures: added source_chip: String param; transfer_effect: chip_name param documented
- `docs/architecture/ordering.md` — BoltSystems::WallCollision: added to Defined Sets table
- `docs/architecture/plugins.md` — ShieldActive cross-domain write: new "ShieldActive Cross-Domain Write Exception" section added; "debug only" claim in rule sentence updated
- `docs/architecture/content.md` — Unified Effect Model: complete replacement of stale Effect enum (ChainHit, ActiveEffects, PiercingApplied, flat passive components, wrong Attraction/Shield/SecondWind/EntropyEngine signatures) with current EffectKind model

**Items confirmed no-drift:**
- `docs/design/effects/shield.md` — matches shield.rs behavior exactly
- `docs/design/effects/chain_lightning.md` — arc_speed omitted from parameter table (see known-state for decision)

## 2026-03-28 — feature/stat-effects review (merged to develop)

**Branch:** feature/stat-effects (merge commit 74d538b)

**Files reviewed:**
- `breaker-game/src/effect/sets.rs` — confirmed both Bridge and Recalculate variants
- `breaker-game/src/effect/plugin.rs` — confirmed Recalculate.after(Bridge) configure_sets
- `breaker-game/src/effect/mod.rs` — confirmed Effective* re-exports
- `breaker-game/src/effect/effects/damage_boost.rs`, `speed_boost.rs`, `size_boost.rs`, `piercing.rs`, `bump_force.rs`, `quick_stop.rs` — confirmed Active*/Effective* pattern
- `breaker-game/src/chips/components.rs` — confirmed intentional stub
- `breaker-game/src/bolt/components.rs` — confirmed PiercingRemaining lives here
- `breaker-game/src/bolt/queries.rs` — confirmed reads EffectivePiercing, EffectiveDamageMultiplier
- `breaker-game/src/breaker/queries.rs` — confirmed reads EffectiveSpeedMultiplier, EffectiveSizeMultiplier
- `breaker-game/src/bolt/sets.rs` — confirmed BoltSystems::CellCollision variant
- `breaker-game/src/breaker/sets.rs` — confirmed BreakerSystems::UpdateState variant
- `breaker-game/src/bolt/plugin.rs` — confirmed prepare_bolt_velocity.after(Recalculate)
- `breaker-game/src/breaker/plugin.rs` — confirmed move_breaker.after(Recalculate)

**Docs updated:**
- `docs/architecture/plugins.md` — 3 changes (Cross-Domain Read Access, EffectSystems entry, sets.rs line in Effect Domain section)
- `docs/architecture/ordering.md` — 3 changes (Defined Sets table additions, FixedUpdate chain Recalculate insertion, Reading narrative)
- `docs/architecture/data.md` — 1 addition (Active/Effective Component Pattern section)
- `docs/plan/index.md` — 1 addition (Stat Effects entry in Current section)

**Items confirmed no-drift:**
- `chips/components.rs` is intentionally a stub — correct, not a missing file
- `effect/effects/` module list in plugins.md is marked "(~24 total)" — non-exhaustive by design
