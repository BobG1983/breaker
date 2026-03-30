---
name: known-state
description: Confirmed doc/code alignment state; covers effect system rewrite (2026-03-28), stat-effects phase (feature/stat-effects), and file-split refactor (2026-03-30)
type: project
---

## Confirmed Correct (as of runtime-effects branch, 2026-03-28)

- `docs/design/effects/explode.md` — "Not yet implemented" removed; explode is fully implemented
- `docs/architecture/messages.md` — DamageCell sender list now includes all effect senders (shockwave, explode, pulse, chain_lightning, piercing_beam, tether_beam). SpawnChainBolt removed (never existed). SpawnAdditionalBolt removed entirely from both code and docs.
- `docs/architecture/ordering.md` — spawn_additional_bolt and spawn_chain_bolt entries removed (neither system exists; effects spawn directly via &mut World)
- `docs/design/terminology/core.md` — ChainBolt entry corrected (now references ChainBoltMarker, ChainBoltAnchor, ChainBoltConstraint, DistanceConstraint; removed SpawnChainBolt/spawn_chain_bolt/break_chain_on_bolt_lost which never existed)
- `docs/plan/index.md` — Runtime Effects entry marked Done (updated 2026-03-30 full verification)
- `docs/architecture/effects/core_types.md` — EffectKind enum is complete and current for all 25 effect modules
- `docs/design/effects/` — all 25 design docs match implemented behavior

## SpawnAdditionalBolt — REMOVED (feature/runtime-effects)

`SpawnAdditionalBolt` was removed from `bolt/messages.rs` and from `docs/architecture/messages.md`.
`spawn_bolts::fire()` and `chain_bolt::fire()` spawn directly via `&mut World`. Do NOT flag its
absence as missing functionality — direct World spawning is the established pattern.

## Confirmed Correct (as of effect system rewrite, 2026-03-28)

- `docs/architecture/messages.md` — Collision messages use `BoltImpactCell`, `BoltImpactWall`, `BreakerImpactCell`, `BreakerImpactWall`, `CellImpactWall`. `DamageCell.source_chip` (not `source_bolt`). Observer Events section replaced with Effect Dispatch section.
- `docs/architecture/effects/core_types.md` — `EffectKind` enum includes `Explode`, `QuickStop`, `TetherBeam`. `SecondWind` is unit variant. `EntropyEngine` uses `max_effects: u32`.
- `docs/architecture/effects/reversal.md` — Passive buffs table uses correct variants. Fire-and-forget category added.
- `docs/architecture/effects/node_types.md` — `Once` example uses `Do(SecondWind)` (unit variant).
- `docs/architecture/layout.md` — Effect domain layout reflects `core/types.rs` + per-trigger-type files in `triggers/`.
- `docs/design/chip-catalog.md` — SpawnBolts correct. No TiltControl/MultiBolt.

## Confirmed Correct (as of stat-effects merge, 2026-03-28+)

- `docs/architecture/plugins.md` — Cross-Domain Read Access: bolt reads `PiercingRemaining` (bolt domain) + `EffectivePiercing`/`EffectiveDamageMultiplier` (effect domain); breaker reads `EffectiveSpeedMultiplier`/`EffectiveSizeMultiplier` (effect domain); cells receives pre-computed damage via `DamageCell` message (no direct Effective* read). `EffectSystems` entry lists both `Bridge` and `Recalculate`. Effect domain `sets.rs` line updated.
- `docs/architecture/ordering.md` — `EffectSystems::Recalculate`, `BoltSystems::CellCollision`, `BreakerSystems::UpdateState` in Defined Sets table. FixedUpdate chain shows Recalculate above Move/PrepareVelocity.
- `docs/architecture/data.md` — Active/Effective Component Pattern section added.
- `docs/plan/index.md` — Stat Effects entry added to Current section.

## Key Architectural Fact: DamageCell pre-bakes multiplier

`handle_cell_hit` (cells domain) does NOT read `EffectiveDamageMultiplier` directly. `bolt_cell_collision` reads it and applies it when computing `effective_damage = BASE_BOLT_DAMAGE * multiplier` — that pre-computed value goes into the `DamageCell.damage` field. Cells are decoupled from the effect stat model.

**Why:** The bolt domain owns collision, so it applies the multiplier at collision time. The cells domain only needs to know how much damage to apply.

**How to apply:** Do not flag cells reading Effective* types as missing — it's correct that cells doesn't read them.

## Intentionally Forward-Looking (do NOT flag as drift)

- `docs/design/chip-catalog.md` — Chip RON files now exist under `breaker-game/assets/chips/` (34+ templates). The doc's additive format vs RON multiplicative format divergence is a known blocker (guard-game-design evaluation-full-verification-2026-03-30.md). Do NOT flag RON file existence as missing.
- `docs/design/effects/ramping_damage.md` — `damage_per_trigger` is correct per code.
- Evolution chips (Entropy Engine, Nova Lance, etc.) — Not yet implemented in code. Design spec only.
- `docs/plan/index.md` — Spatial/Physics Extraction and Stat Effects are both correctly marked Done.

## chips/components.rs — Intentional Stub

The file contains only a doc comment explaining legacy stat components were removed. Do not flag as a missing/empty file. Chip stat components (DamageBoost, BoltSpeedBoost, BreakerSpeedBoost, BumpForceBoost, Piercing) were removed; state is now managed by effect domain Active*/Effective* pairs.

**Why:** stat-effects phase migration removed all flat chip stat components.
**How to apply:** When reviewing chips domain, expect components.rs to be a stub with doc comment only.

## Architecture Confirmed (source-chip-shield-absorption, 2026-03-29)

- Effect dispatch: `EffectKind::fire(entity, source_chip: &str, world)` / `reverse(entity, source_chip: &str, world)` — source_chip added to ALL fire/reverse signatures
- `EffectCommandsExt` methods: `fire_effect(entity, effect, source_chip: String)`, `reverse_effect(entity, effect, source_chip: String)`, `transfer_effect(entity, chip_name: String, children, permanent)`, `push_bound_effects(entity, effects: Vec<(String, EffectNode)>)`
- `PushBoundEffects` — custom `Command` struct in `effect/commands.rs`; inserts `BoundEffects`+`StagedEffects` if absent, appends entries. Used by `dispatch_cell_effects` and `dispatch_breaker_effects`.
- `CellEffectsDispatched` — marker component in `cells/components/types.rs`; prevents double-dispatch by `dispatch_cell_effects`
- `dispatch_cell_effects` — cells system; `OnEnter(GameState::Playing)` after `NodeSystems::Spawn`; skips cells with `CellEffectsDispatched`
- `dispatch_breaker_effects` — breaker system; `OnEnter(GameState::Playing)` chained after `init_breaker`, both after `BreakerSystems::InitParams` and `NodeSystems::Spawn`
- `dispatch_wall_effects` — wall system; `OnEnter(GameState::Playing)` chained after `spawn_walls`; currently a no-op stub (walls have no RON-defined effects)
- `ChainArcCountReasonable` — new `InvariantKind` variant; checks combined `ChainLightningChain` + `ChainLightningArc` entity count against `invariant_params.max_chain_arc_count` (default 50)
- `SpawnExtraChainArcs(usize)` — new `MutationKind` variant; spawns N chain + N arc entities for self-test
- InvariantKind total: 25 variants (22 + `AabbMatchesEntityDimensions`, `GravityWellCountReasonable`, `SizeBoostInRange` added in feature/missing-unit-tests)
- MutationKind total: 18 variants (`InjectMismatchedBoltAabb`, `SpawnExtraGravityWells`, `InjectWrongSizeMultiplier` added in feature/missing-unit-tests)
- `EffectSourceChip(Option<String>)` — component on AoE/spawn effect entities; carries chip attribution from dispatch to damage-application tick
- `chip_attribution(source_chip: &str) -> Option<String>` — helper: empty → None, non-empty → Some
- fire() method split: `fire` → `fire_aoe_and_spawn` → `fire_utility_and_spawn` (3 methods)
- reverse() method split: `reverse` → `reverse_aoe_and_spawn` (2 methods)
- `EffectKind::Shield { stacks: u32 }` — only field is stacks (NO base_duration, NO duration_per_level)
- `EffectKind::Attraction { attraction_type, force, max_force: Option<f32> }` — named fields (NOT tuple)
- `EffectKind::ChainLightning { arcs, range, damage_mult, arc_speed }` — arc_speed field (default 200.0 via serde)
- `EffectKind::Pulse { base_range, range_per_level, stacks, speed, interval }` — interval field (default 0.5 via serde)
- `BoltSystems::WallCollision` — defined in bolt/sets.rs, tags bolt_wall_collision, runs after CellCollision
- ShieldActive cross-domain writes: bolt domain writes ShieldActive on breaker entity (bolt_lost); cells domain writes ShieldActive on cell entities (handle_cell_hit). Both are accepted architectural exceptions.
- No typed observer events. No `ActiveEffects`/`ArmedEffects`/`EffectChains` resources.
- Chain stores: `BoundEffects` (permanent) + `StagedEffects` (one-shot)
- Effect file pattern: `fire(entity, ..params.., source_chip: &str, world)` + `reverse(...)` + `register()` free functions per module
- Stat model: `Active*` stacks (effect domain) → `Effective*` scalars (computed by Recalculate) → consumers
- `PiercingRemaining` is bolt domain (gameplay state), not an effect stat. `EffectivePiercing` is the cap.
- `EffectSystems::Recalculate` ordering: `.after(EffectSystems::Bridge)`, run_if `in_state(PlayingState::Active)`

## Confirmed Correct / Fixed (file-split refactor, 2026-03-30)

- `effect/core/types.rs` is now `effect/core/types/` directory module (`mod.rs` + `definitions.rs` + `tests.rs`). All docs updated: `core_types.md`, `layout.md`, `plugins.md`, `structure.md`, `adding_effects.md`, `adding_triggers.md`, `content.md`.
- Many effect modules are now directory modules (shockwave/, chain_bolt/, chain_lightning/, explode/, tether_beam/, pulse/, piercing_beam/, attraction/, spawn_bolts/, spawn_phantom/, entropy_engine/, second_wind/, random_effect/). Layout docs updated.
- Trigger modules evaluate/, impact/, impacted/, until/ are now directory modules. Layout docs updated.
- `EffectChains`, `ActiveEffects`, `ArmedEffects` — removed from `chips.md` terminology (old architecture). Replaced with `BoundEffects` and `StagedEffects` entries.
- `OnSelected` trigger — never existed as a code variant. Removed from chips.md. Correct pattern is `On(target: X, then: [...])` via `RootEffect::On`. `dispatch.md` already documents this correctly.
- `OnBump` — removed from chips.md; correct variant is `Bump`.
- `OnPerfectBump` → `PerfectBump` in When example in chips.md.
- `EffectNode` node count: updated to 6 (was incorrectly stated as 4).
- `Target::AllWalls` — added to chips.md; was missing from the variant list.
- `EvolutionRegistry` → `EvolutionTemplateRegistry` in plugins.md and plan/index.md.
- `ChipRegistry` → `ChipTemplateRegistry`/`ChipCatalog` in content.md registries section.
- RON examples in chip-rarity-rework.md, chip-template-system.md, content.md updated to use `On(target: X, then: [...])` notation.
- `EffectChains` references in evolutions.md replaced with `BoundEffects`.
- `RootEffect` entry in chips.md: updated file reference from `effect/definition.rs` to `effect/core/types/definitions.rs`.
- `plan/index.md`: Runtime Effects updated from "In Progress" to "Done" (all 24 effects implemented and merged).

## RON Format Confirmed (2026-03-30)

- Chip template fields: `common:`, `uncommon:`, `rare:`, `legendary:` — NOT `Some((...))`; absence means the slot is not present
- Effect dispatch in RON: `On(target: Bolt, then: [Do(Piercing(1))])` — top-level wrapper is `RootEffect::On`, not `When(trigger: OnSelected, ...)`
- Trigger chip names: `DamageBoost(1.1)` not `DamageBoost(0.1)` for the rare Piercing chip example
