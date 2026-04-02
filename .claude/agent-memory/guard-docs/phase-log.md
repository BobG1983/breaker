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

## 2026-03-30 — Full Verification Tier doc check on develop

**Branch:** develop (commit c9964b7 refactor: split 23 oversized .rs files into directory modules)

**Files reviewed:**
- `breaker-game/src/effect/core/types/` — confirmed directory module split
- `breaker-game/src/effect/effects/` — confirmed multiple directory module splits
- `breaker-game/src/effect/triggers/evaluate/`, `impact/`, `impacted/`, `until/` — confirmed directory module splits
- `breaker-game/src/effect/effects/gravity_well.rs`, `second_wind/system.rs` — confirmed all effects implemented
- `breaker-game/assets/chips/templates/piercing.chip.ron`, `augment.chip.ron` — confirmed RON syntax
- `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs` — confirmed no OnSelected trigger
- `breaker-game/src/effect/core/types/definitions.rs` — confirmed Trigger enum (no OnSelected/OnBump)
- `breaker-game/src/game.rs` — confirmed plugin order unchanged

**Drifts found and fixed:**
- `docs/plan/index.md` — Runtime Effects: "In Progress" → "Done" (all 24 effects implemented and merged)
- `docs/plan/index.md` — EvolutionRegistry → EvolutionTemplateRegistry
- `docs/architecture/effects/core_types.md` — "All core types live in effect/core/types.rs" → types/ directory
- `docs/architecture/effects/core_types.md` — EffectSourceChip location updated; Per-Effect Modules expanded to include dir modules
- `docs/architecture/layout.md` — effect domain tree updated: core/types/, effect dir modules, trigger dir modules; rule updated to "one module per effect"
- `docs/architecture/plugins.md` — "Actual Structure" block updated to reflect dir modules; EvolutionRegistry → EvolutionTemplateRegistry
- `docs/architecture/effects/structure.md` — domain tree updated (types/, commands.rs + PushBoundEffects, dir modules notation)
- `docs/architecture/effects/adding_effects.md` — types.rs → types/definitions.rs
- `docs/architecture/effects/adding_triggers.md` — types.rs → types/definitions.rs
- `docs/architecture/effects/trigger_systems.md` — impact.rs/impacted.rs → impact//impacted/ directory modules
- `docs/architecture/content.md` — types.rs → types/definitions.rs (x2); RON example updated to current syntax; ChipRegistry/EvolutionRegistry → ChipTemplateRegistry/ChipCatalog/EvolutionTemplateRegistry
- `docs/design/terminology/chips.md` — removed stale EffectChains/ActiveEffects/ArmedEffects; replaced with BoundEffects/StagedEffects; removed OnSelected/OnBump; fixed OnPerfectBump → PerfectBump; fixed Until field name; added AllWalls to Target list; updated EffectNode count to 6; updated RootEffect file reference
- `docs/design/evolutions.md` — EffectChains → BoundEffects (x2)
- `docs/design/decisions/chip-template-system.md` — RON example updated to current syntax; slot description updated
- `docs/design/decisions/chip-rarity-rework.md` — RON examples updated to current syntax

**Items confirmed no-drift:**
- `docs/architecture/messages.md` — unchanged; correct
- `docs/architecture/ordering.md` — unchanged; correct
- `docs/architecture/effects/dispatch.md` — already correctly documents no OnSelected trigger
- `docs/architecture/effects/evaluation.md` — correct
- `docs/architecture/effects/commands.md` — correct
- game.rs plugin registration order — unchanged

## 2026-03-30 — develop: new invariants and mutations drift fix

**Branch:** develop (commit fad7dfa — feature/missing-unit-tests merged; 58 unit tests + 3 new invariants)

**Files reviewed:**
- `breaker-scenario-runner/src/types/definitions/invariants.rs` — confirmed 25 variants in ALL
- `breaker-scenario-runner/src/types/definitions/mutations.rs` — confirmed 18 MutationKind variants
- `breaker-scenario-runner/src/types/tests/invariant_kinds.rs` — count test asserts 25
- `breaker-scenario-runner/scenarios/chaos/` — confirmed chaos/ directory exists on disk

**Drifts found and fixed:**
- `docs/architecture/standards.md` — invariant list in "Scenario Coverage" section: added AabbMatchesEntityDimensions, GravityWellCountReasonable, SizeBoostInRange (22 → 25)
- `docs/architecture/standards.md` — scenario categories list: added chaos/ directory
- `docs/architecture/standards.md` — coverage manifest line: "All 22 invariants" → "All 25 invariants"
- `docs/architecture/standards.md` — Scenario Runner "Invariants checked each frame" list: added same 3 variants
- `docs/design/terminology/scenarios.md` — MutationKind list: added InjectMismatchedBoltAabb, SpawnExtraGravityWells, InjectWrongSizeMultiplier (15 → 18 variants)

**Items confirmed no-drift:**
- `docs/plan/index.md` — no plan entry for individual invariant additions; no update needed

## 2026-03-30 — feature/missing-unit-tests review

**Branch:** feature/missing-unit-tests (58 new unit tests + overlay_color pub(super) visibility change)

**Files reviewed:**
- `breaker-game/src/fx/transition/system.rs` — overlay_color fn, TransitionConfig, TransitionStyle, TransitionDirection
- `breaker-game/src/fx/transition/tests.rs` — 10 tests covering spawn/animate/cleanup/color helpers
- `docs/architecture/standards.md` — test placement description
- `docs/architecture/layout.md` — System File Split Convention
- `docs/plan/index.md` — phase completion status
- `docs/architecture/messages.md` — unchanged
- `docs/architecture/plugins.md` — unchanged
- `docs/architecture/ordering.md` — unchanged
- `docs/architecture/data.md` — unchanged
- `docs/design/graphics/` — working-tree modifications confirmed consistent (catalog/ exists, index.md links to it)

**Drifts found and fixed:**
- `docs/architecture/standards.md` line 33 — "Tests live next to the code they test (in-module `#[cfg(test)]` blocks)." → updated to acknowledge both in-module AND split-file patterns (split is the dominant form per layout.md System File Split Convention)

**Items confirmed no-drift:**
- `overlay_color` visibility change (pub(super)) — not a doc-level concern; internal helper not referenced in any architecture doc
- 58 new tests — all follow split-file pattern already documented in layout.md; no new behaviors, systems, or components introduced
- `docs/plan/index.md` — phase completion status accurate; no new phases
- `docs/design/graphics/index.md` modification — references catalog/ which exists on disk; consistent
- `docs/design/graphics/effects-particles.md` modification — working-tree change; design doc (forward-looking Phase 5 content), not architecture drift

## 2026-03-30 — feature/scenario-coverage Effective* cache removal review

**Branch:** feature/scenario-coverage

**Scope:** Checked all docs for references to deleted Effective* cache components and EffectSystems::Recalculate after the refactor that removed all 6 Effective* cache components, their recalculate systems, and 2 invariant checkers (SizeBoostInRange + InjectWrongSizeMultiplier).

**Files reviewed:**
- `breaker-game/src/effect/sets.rs` — confirmed only Bridge variant remains
- `breaker-game/src/effect/effects/speed_boost.rs`, `damage_boost.rs`, `size_boost.rs`, `piercing.rs` — confirmed no Effective* types, no recalculate_* systems
- `breaker-scenario-runner/src/types/definitions/invariants.rs` — confirmed 23 variants (SizeBoostInRange removed)
- `breaker-scenario-runner/src/types/definitions/mutations.rs` — confirmed InjectWrongSizeMultiplier removed
- `docs/architecture/data.md`, `docs/architecture/plugins.md`, `docs/architecture/ordering.md`

**Drifts found and fixed:**
- `docs/architecture/data.md` — "Active/Effective Component Pattern" section rewrote to "Active Component Pattern" (removed Effective* cache description, EffectSystems::Recalculate reference, EffectivePiercing as cap; updated to direct-read model with .multiplier()/.total())
- `docs/architecture/plugins.md` — Effect File Pattern code snippet: removed `recalculate_speed` from register(); added `_source_chip: &str` to fire()/reverse() signatures; added `.multiplier()` method; updated comment from "adds recalculation system" to "wires app systems"
- `docs/architecture/effects/core_types.md` — Per-Effect Modules section: replaced `app.add_systems(FixedUpdate, recalculate_speed)` in register() body with a comment explaining simple stat effects have no runtime systems

**Items confirmed no-drift (already updated by team before this session):**
- `docs/architecture/ordering.md` — EffectSystems::Recalculate already removed from Defined Sets table; Reading paragraph already says "Consumers read Active* components directly via .multiplier()/.total() methods"
- `docs/architecture/plugins.md` — EffectSystems variants entry already shows `Bridge` only (no Recalculate)
- `docs/architecture/standards.md` — invariant list already correct (23 invariants, no SizeBoostInRange)
- `docs/architecture/plugins.md` — Cross-Domain Read Access section already references ActiveDamageBoosts/ActiveSpeedBoosts/ActiveSizeBoosts/ActivePiercings (not Effective* types)

## 2026-03-30 — feature/scenario-coverage Wave 3 review

**Branch:** feature/scenario-coverage

**Files reviewed:**
- `breaker-game/src/effect/effects/tether_beam/effect.rs` — confirmed chain: bool field, TetherChainBeam, TetherChainActive, maintain_tether_chain
- `breaker-game/src/effect/core/types/definitions/enums.rs` — full EffectKind enum (all variants incl. Wave 3)
- `breaker-game/src/effect/core/types/definitions/fire.rs` — 4 fire methods
- `breaker-game/src/effect/core/types/definitions/reverse.rs` — 4 reverse methods
- `breaker-game/src/effect/effects/mod.rs` — confirmed anchor, circuit_breaker, mirror_protocol, flash_step registered
- `breaker-game/src/effect/effects/flash_step.rs` — single file
- `breaker-game/src/effect/effects/anchor/mod.rs` — directory module
- `breaker-game/src/effect/effects/circuit_breaker/mod.rs` — directory module
- `breaker-game/src/effect/effects/mirror_protocol/mod.rs` — directory module
- `breaker-game/src/effect/effects/spawn_bolts/effect.rs` — confirmed inherit copies from primary bolt
- `docs/design/effects/tether_beam.md` — chain field and chain mode documented correctly
- `docs/design/effects/flash_step.md`, `mirror_protocol.md`, `anchor.md`, `circuit_breaker.md`, `spawn_bolts.md` — confirmed correct
- `breaker-scenario-runner/src/types/definitions/invariants.rs` — confirmed 25 variants (no new ones in Wave 3)

**Drifts found and fixed:**
- `docs/architecture/effects/core_types.md` — TetherBeam variant: `{ damage_mult: f32 }` → added `chain: bool` field
- `docs/architecture/effects/core_types.md` — EffectKind block: added FlashStep, MirrorProtocol, Anchor, CircuitBreaker variants
- `docs/architecture/effects/core_types.md` — fire/reverse method count: "3 fire / 2 reverse" → "4 fire / 4 reverse" (fire_breaker_effects and reverse_utility/reverse_breaker_effects added)
- `docs/architecture/effects/core_types.md` — opening line: `definitions.rs` → `definitions/enums.rs`; EffectSourceChip location updated
- `docs/architecture/effects/structure.md` — `definitions.rs` → `definitions/` directory module with enums.rs/fire.rs/reverse.rs
- `docs/architecture/effects/adding_effects.md` — `definitions.rs` → `definitions/enums.rs`; step 3 updated to name fire.rs/reverse.rs
- `docs/architecture/effects/adding_triggers.md` — `definitions.rs` → `definitions/enums.rs`
- `docs/architecture/layout.md` — types/ tree updated: definitions.rs → definitions/ directory; flash_step.rs + anchor/circuit_breaker/mirror_protocol/ added; rule updated
- `docs/architecture/plugins.md` — Actual Structure block: definitions.rs → definitions/ directory; flash_step.rs + anchor/circuit_breaker/mirror_protocol/ added
- `docs/architecture/content.md` — TetherBeam variant: added chain field; added FlashStep/MirrorProtocol/Anchor/CircuitBreaker variants; both definitions.rs references → definitions/enums.rs
- `docs/design/terminology/chips.md` — BoundEffects/StagedEffects/RootEffect: `definitions.rs` → `definitions/enums.rs`

**Items confirmed no-drift:**
- `docs/design/effects/tether_beam.md` — chain field and chain/standard mode sections already present (correct)
- `docs/design/effects/spawn_bolts.md` — inherit field already documented
- InvariantKind count — still 25 (no new invariants in Wave 3)
- `docs/architecture/ordering.md` — maintain_tether_chain is effect-internal (no cross-domain ordering); no update needed
- `docs/plan/index.md` — no plan entry for Wave 3 scenario coverage; no update needed

## 2026-04-01 — feature/breaker-builder-pattern Wave 9 doc update

**Branch:** feature/breaker-builder-pattern (Waves 1-8 complete)

**Files reviewed:**
- `breaker-game/src/breaker/builder/core.rs` — 7 typestate dimensions, 4 terminal impls
- `breaker-game/src/breaker/plugin.rs` — spawn_or_reuse_breaker, no BreakerConfig
- `breaker-game/src/breaker/definition.rs` — 36+ fields with serde defaults
- `breaker-game/src/bolt/builder/core.rs` — 6 dimensions, BaseRadius/MinRadius/MaxRadius
- `breaker-game/src/bolt/definition.rs` — BoltDefinition with min_radius/max_radius
- `breaker-game/src/bolt/components/definitions.rs` — BoltRadius = type alias for BaseRadius
- `breaker-game/assets/breakers/breaker.example.ron` — confirmed exists (plan called for .template.ron)
- `breaker-game/assets/bolts/bolt.example.ron` — confirmed exists

**Docs updated:**
- `docs/architecture/builders/pattern.md` — 7 changes: BreakerBuilder 7th R param, transition method signatures, `.config` → `.definition`, terminal impl, 4 terminal blocks note, conventions, current implementations table
- `docs/architecture/builders/breaker.md` — full rewrite: 7 dimensions, eliminated renames, correct component names, 4 terminal blocks, spawn_or_reuse_breaker, retained systems updated
- `docs/architecture/builders/bolt.md` — 4 changes: BaseRadius+MinRadius+MaxRadius in build() output, BoltRadius alias note, `.config` → `.definition`, key files updated
- `docs/architecture/data.md` — 6 changes: pipeline rewrite (Registry→Builder→Entity), BreakerConfig eliminated section, BumpVisualParams → BumpFeedback example, init_breaker_params→builder pattern, extensions bdef.ron→breaker.ron (x2)
- `docs/architecture/plugins.md` — 1 change: BreakerWidth/BreakerHeight → BaseWidth/BaseHeight in cross-domain read access
- `docs/architecture/bolt-definitions.md` — 5 changes: status banner added, "BoltConfig eliminated" section, spawn flow updated, bolt-lost updated, RON filenames .bdef.ron → .breaker.ron, Breaker Definition Changes rewritten
- `docs/design/graphics/gameplay-elements.md` — 1 change: EntityScale → NodeScalingFactor

**Items confirmed no-drift:**
- `docs/architecture/layout.md` — no EntityScale references remain
- `docs/architecture/content.md` — no BreakerConfig or .bdef.ron references
- `plan/index.md` — active work tracked in todos, no phase entry needed
- RON templates: `breaker.example.ron` and `bolt.example.ron` already exist with correct content; plan called for `.template.ron` but `.example.ron` was implemented — names differ but content is correct

## 2026-04-01 — feature/chip-evolution-ecosystem steering model + gravity_well split review

**Branch:** feature/chip-evolution-ecosystem (commits: c007143 attraction/gravity-well steering, 9e7f476 bolt typestate builder)

**Files reviewed:**
- `breaker-game/src/effect/effects/speed_boost.rs` — recalculate_velocity added to fire/reverse
- `breaker-game/src/effect/effects/attraction/effect.rs` — apply_attraction with apply_velocity_formula
- `breaker-game/src/effect/effects/gravity_well/effect.rs` — apply_gravity_pull, confirmed directory module
- `breaker-scenario-runner/src/types/definitions/invariants.rs` — 23 variants, BoltSpeedAccurate
- `breaker-scenario-runner/src/types/definitions/mutations.rs` — 16 variants confirmed
- `docs/architecture/rendering/` — all forward-looking Phase 5 design docs
- `docs/plan/phase-5/` — planning docs, not yet implemented

**Drifts found and fixed:**
- `docs/architecture/layout.md` — `gravity_well.rs` → `gravity_well/` (directory module)
- `docs/architecture/plugins.md` — `gravity_well.rs` in two places → `gravity_well/` and path to `effect.rs`
- `docs/architecture/plugins.md` — Velocity2D exception: "two systems" → "three write paths"; added speed_boost::fire()/reverse() path
- `docs/architecture/plugins.md` — Effect File Pattern: fire/reverse comments updated to mention recalculate_velocity
- `docs/architecture/effects/core_types.md` — Per-Effect Modules fire() comment: added recalculate_velocity note
- `docs/architecture/standards.md` — `BoltSpeedInRange` → `BoltSpeedAccurate` (×2: invariant list and scenario runner list)
- `docs/design/terminology/scenarios.md` — MutationKind: removed `InjectWrongEffectiveSpeed` and `InjectWrongSizeMultiplier` (both deleted from code)

**Items confirmed no-drift:**
- InvariantKind total: 23 (unchanged)
- `docs/plan/index.md` — no phase completion changes needed
- All Phase 5 rendering docs — intentionally forward-looking
- `docs/architecture/bolt-definitions.md` — forward-looking sections unchanged

## 2026-04-02 — feature/breaker-builder-pattern Wave 9 final doc check

**Branch:** feature/breaker-builder-pattern (Wave 9)

**Files reviewed:**
- `breaker-game/src/bolt/plugin.rs` — dispatch_bolt_effects placement in FixedUpdate
- `breaker-game/src/bolt/builder/core.rs` — spawn() signature (World not Commands)
- `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs` — Added<BoltDefinitionRef> trigger
- `breaker-game/src/bolt/systems/sync_bolt_scale.rs` — system name confirmed
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs` — spawn_bolt(world: &mut World) confirmed
- `breaker-game/src/breaker/sets.rs` — BreakerSystems variants confirmed (no InitParams)
- `breaker-game/src/bolt/sets.rs` — BoltSystems variants confirmed
- `breaker-game/src/breaker/queries.rs` — QueryData struct names confirmed
- `breaker-game/src/breaker/plugin.rs` — no spawn_lives_display
- `breaker-game/src/ui/plugin.rs` — spawn_timer_hud only (no spawn_lives_display)
- `breaker-game/src/effect/commands/ext.rs` — transfer_effect signature (5 params with TriggerContext)
- `breaker-game/src/effect/core/types/definitions/enums.rs` — TriggerContext struct confirmed

**Drifts found and fixed:**
- `docs/architecture/ordering.md` — removed non-existent `spawn_lives_display` from OnEnter chain
- `docs/architecture/ordering.md` — added `apply_node_scale_to_bolt` to OnEnter chain (was missing)
- `docs/architecture/ordering.md` — added `dispatch_bolt_effects` to FixedUpdate chain
- `docs/architecture/ordering.md` — added `cleanup_destroyed_bolts.after(EffectSystems::Bridge)` to FixedUpdate chain
- `docs/architecture/ordering.md` — added `.after(EnforceDistanceConstraints)` to bolt_cell_collision in MaintainQuadtree section
- `docs/architecture/messages.md` — transfer_effect: added `context: TriggerContext` 5th param
- `docs/architecture/plugins.md` — transfer_effect: added `context: TriggerContext` 5th param
- `docs/architecture/builders/breaker.md` — Key Files: `BreakerBumpData` → `BreakerBumpTimingData` + `BreakerBumpGradingData` + `SyncBreakerScaleData`
- `docs/architecture/builders/bolt.md` — spawn() Behavior: corrected to `&mut World` (not `&mut Commands`); removed dispatch_initial_effects claim
- `docs/architecture/builders/pattern.md` — Output Paths table: noted spawn() takes World for Bolt vs Commands for Breaker
- `docs/architecture/bolt-definitions.md` — step 5: clarified dispatch_bolt_effects runs in FixedUpdate not OnEnter
- `docs/design/terminology/core.md` — BreakerState → DashState; BoltSpeed → BaseSpeed in code examples
- `docs/architecture/data.md` — BoltBaseSpeed/BoltRadius example: replaced with BoltBaseDamage/BoltSpawnOffsetY; BoltRadius alias note

**Items confirmed no-drift:**
- `docs/architecture/messages.md` — BreakerSpawned sender correct (spawn_or_reuse_breaker)
- `docs/architecture/layout.md` — builder/ module not documented (by design — see pattern.md)
- `docs/architecture/standards.md` — invariant list unchanged (23 variants)
- `docs/plan/index.md` — builder pattern is infra work, not a plan milestone

## 2026-03-31 — feature/chip-evolution-ecosystem bolt builder migration review

**Branch:** feature/chip-evolution-ecosystem

**Scope:** Checked all docs for drift from bolt builder migration: `init_bolt_params` deleted, `spawn_extra_bolt` removed from `fire_helpers.rs`, `prepare_bolt_velocity` deleted, `BoltSystems::InitParams` removed, `MaxReflectionAngle` → `BreakerReflectionSpread`.

**Files reviewed:**
- `breaker-game/src/bolt/sets.rs` — confirmed no InitParams variant
- `breaker-game/src/bolt/systems/mod.rs` — confirmed no init_bolt_params, no prepare_bolt_velocity
- `breaker-game/src/bolt/builder.rs` — confirmed Bolt::builder() typestate, 5 dimensions, config() method
- `breaker-game/src/bolt/plugin.rs` — confirmed spawn_bolt + reset_bolt only (no init_bolt_params)
- `breaker-game/src/bolt/resources.rs` — confirmed BoltConfig still exists
- `breaker-game/src/bolt/components/definitions.rs` — confirmed PrimaryBolt new, BoltRespawnOffsetY/BoltRespawnAngleSpread/BoltInitialAngle still exist
- `breaker-game/src/bolt/queries.rs` — confirmed apply_velocity_formula, no separate prepare step
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs` — confirmed Bolt::builder() usage
- `breaker-game/src/bolt/systems/bolt_lost/system.rs` — confirmed reads BoltRespawnOffsetY/BoltRespawnAngleSpread from entity
- `breaker-game/src/bolt/systems/launch_bolt.rs` — confirmed reads BoltInitialAngle component
- `breaker-game/src/breaker/components/core.rs` — confirmed BreakerReflectionSpread (was MaxReflectionAngle)
- `breaker-game/src/effect/effects/fire_helpers.rs` — confirmed spawn_extra_bolt removed; only entity_position + effective_range
- `breaker-game/src/effect/effects/mod.rs` — confirmed no spawn_extra_bolt export
- `breaker-game/src/effect/effects/spawn_bolts/effect.rs` — confirmed direct Bolt::builder() call

**Drifts found and fixed:**
- `docs/architecture/data.md` — `MaxReflectionAngle` → `BreakerReflectionSpread` (x3); `Without<BreakerMaxSpeed>` → `Without<MaxSpeed>`
- `docs/architecture/layout.md` — `effects/mod.rs` description: removed stale `spawn_extra_bolt helper`
- `docs/architecture/plugins.md` — `effects/mod.rs` description: removed stale `spawn_extra_bolt helper`
- `docs/architecture/effects/structure.md` — `effects/mod.rs` line: removed stale `spawn_extra_bolt helper`
- `docs/architecture/bolt-definitions.md` — "Current State" section: spawn flow, extra bolt spawn, bolt-lost, breaker→bolt relationship all updated for builder migration; "Not in BoltDefinition" `init_bolt_params` → builder; misleading "BoltConfig is eliminated entirely" → "Target:"

**Items confirmed no-drift:**
- `docs/architecture/ordering.md` — `spawn_bolt [uses Bolt::builder()]` already correct; no BoltSystems::InitParams
- `docs/architecture/type_state_builder_pattern.md` — already documents Bolt::builder() correctly
- `docs/plan/index.md` — no plan entry needed (internal refactor, not a phase milestone)
- `docs/architecture/bolt-definitions.md` Target State section — forward-looking, all `init_bolt_params` references there are planned target code
