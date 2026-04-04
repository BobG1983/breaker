---
name: known-state
description: Confirmed doc/code alignment state; covers effect system rewrite (2026-03-28), stat-effects phase (feature/stat-effects), file-split refactor (2026-03-30), and bolt builder migration (2026-03-31)
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

**NOTE: This section describes the stat-effects architecture as of 2026-03-28. The
Effective* cache-removal refactor (2026-03-30) superseded this. See the
"Confirmed Correct / Fixed (Effective* cache removal)" section below for current state.**

- [SUPERSEDED] `docs/architecture/plugins.md` — Referenced `EffectivePiercing`/`EffectiveDamageMultiplier`/`EffectiveSpeedMultiplier`/`EffectiveSizeMultiplier` — ALL REMOVED in cache-removal refactor. Verify against current code.
- [SUPERSEDED] `docs/architecture/ordering.md` — Referenced `EffectSystems::Recalculate` and `BoltSystems::PrepareVelocity` — BOTH REMOVED.
- `docs/architecture/data.md` — Section renamed "Active Component Pattern" (no longer "Active/Effective").
- `docs/plan/index.md` — Stat Effects entry is Done.

## Key Architectural Fact: DamageCell pre-bakes multiplier

`handle_cell_hit` (cells domain) does NOT read damage multipliers directly. `bolt_cell_collision`
reads `ActiveDamageBoosts` (NOT `EffectiveDamageMultiplier` — that type no longer exists) and calls
`.multiplier()` to compute `effective_damage = BASE_BOLT_DAMAGE * mult` — that pre-computed value
goes into the `DamageCell.damage` field. Cells are decoupled from the effect stat model.

**Why:** The bolt domain owns collision, so it applies the multiplier at collision time. The cells domain only needs to know how much damage to apply.

**How to apply:** Do not flag cells reading `Active*` types as missing — it's correct that cells doesn't read them. `EffectiveDamageMultiplier` does NOT exist — removed in Effective* cache-removal refactor.

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
- `PushBoundEffects` — custom `Command` struct in `effect/commands.rs`; inserts `BoundEffects`+`StagedEffects` if absent, appends entries. Used by `dispatch_cell_effects` (cells still uses it); `dispatch_breaker_effects` is SUPERSEDED by `spawn_or_reuse_breaker` builder path.
- `CellEffectsDispatched` — marker component in `cells/components/types.rs`; prevents double-dispatch by `dispatch_cell_effects`
- `dispatch_cell_effects` — cells system; `OnEnter(GameState::Playing)` after `NodeSystems::Spawn`; skips cells with `CellEffectsDispatched`
- `dispatch_breaker_effects` — **SUPERSEDED** in feature/breaker-builder-pattern: replaced by `spawn_or_reuse_breaker` which uses `Breaker::builder()` and fires effects inline. See line ~199 (State after Waves 1-8).
- `dispatch_wall_effects` — **DELETED** in wall-builder-pattern feature. `spawn_walls` now reads from `WallRegistry`, calls `Wall::builder()` three times, and dispatches effects inline via `push_bound_effects`. No separate dispatch system exists.
- `ChainArcCountReasonable` — new `InvariantKind` variant; checks combined `ChainLightningChain` + `ChainLightningArc` entity count against `invariant_params.max_chain_arc_count` (default 50)
- `SpawnExtraChainArcs(usize)` — new `MutationKind` variant; spawns N chain + N arc entities for self-test
- InvariantKind total: 23 variants (25 - 2 removed: `EffectiveSpeedConsistent` and `SizeBoostInRange` removed with Effective* cache removal; `BoltSpeedInRange` renamed to `BoltSpeedAccurate`)
- MutationKind total: verify current count — `InjectWrongSizeMultiplier` and `InjectWrongEffectiveSpeed` removed with Effective* cache removal
- `EffectSourceChip(Option<String>)` — component on AoE/spawn effect entities; carries chip attribution from dispatch to damage-application tick
- `chip_attribution(source_chip: &str) -> Option<String>` — helper: empty → None, non-empty → Some
- fire() method split: `fire` → `fire_aoe_and_spawn` → `fire_utility_and_spawn` (3 methods)
- reverse() method split: `reverse` → `reverse_aoe_and_spawn` (2 methods)
- `EffectKind::Shield { duration: f32 }` — spawns a timed floor wall (`ShieldWall` + `ShieldWallTimer`); NO stacks/charges mechanism; ShieldActive type NO LONGER EXISTS
- `EffectKind::Attraction { attraction_type, force, max_force: Option<f32> }` — named fields (NOT tuple)
- `EffectKind::ChainLightning { arcs, range, damage_mult, arc_speed }` — arc_speed field (default 200.0 via serde)
- `EffectKind::Pulse { base_range, range_per_level, stacks, speed, interval }` — interval field (default 0.5 via serde)
- `BoltSystems::WallCollision` — defined in bolt/sets.rs, tags bolt_wall_collision, runs after CellCollision
- ShieldActive cross-domain writes: ELIMINATED. `ShieldActive` component no longer exists. Shield is now a world-space floor wall (`ShieldWall`) with a timer. The "ShieldActive Cross-Domain Write Exception" section in plugins.md has been removed.
- No typed observer events. No `ActiveEffects`/`ArmedEffects`/`EffectChains` resources.
- Chain stores: `BoundEffects` (permanent) + `StagedEffects` (one-shot)
- Effect file pattern: `fire(entity, ..params.., source_chip: &str, world)` + `reverse(...)` + `register()` free functions per module
- Stat model (AFTER Effective* cache removal): `Active*` stacks → consumers call `.multiplier()` / `.total()` directly. NO `Effective*` components. NO `EffectSystems::Recalculate`. `EffectSystems` has only `Bridge`.
- `PiercingRemaining` is bolt domain (gameplay state), not an effect stat. Cap is `ActivePiercings::total()` (not `EffectivePiercing`).

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

## Confirmed Correct / Fixed (Wave 3 / feature/scenario-coverage, 2026-03-30)

**IMPORTANT: `definitions.rs` was further split into `definitions/` directory module:**
- `effect/core/types/definitions.rs` is now `effect/core/types/definitions/` with `mod.rs` + `enums.rs` + `fire.rs` + `reverse.rs`.
- `enums.rs` holds all types (Trigger, EffectKind, EffectNode, etc.); `fire.rs`/`reverse.rs` hold dispatch methods.
- All docs now reference `effect/core/types/definitions/enums.rs` (not `definitions.rs`).
- Updated: `core_types.md`, `layout.md`, `plugins.md`, `structure.md`, `adding_effects.md`, `adding_triggers.md`, `content.md`, `chips.md`.

**New Wave 3 effects (4 additions to EffectKind):**
- `TetherBeam { damage_mult: f32, #[serde(default)] chain: bool }` — chain field added. Chain mode connects all bolts instead of spawning new ones.
- `FlashStep` — unit variant. Inserts `FlashStepActive` on breaker. Reversal removes it.
- `MirrorProtocol { #[serde(default)] inherit: bool }` — spawns a mirrored bolt based on last impact position/side.
- `Anchor { bump_force_multiplier, perfect_window_multiplier, plant_delay }` — plant mechanic. Components: `AnchorActive`, `AnchorTimer`, `AnchorPlanted`.
- `CircuitBreaker { bumps_required, spawn_count, inherit, shockwave_range, shockwave_speed }` — charge-and-release. Component: `CircuitBreakerCounter`.

**New types in tether_beam module:**
- `TetherChainBeam` — marker on chain-mode beam entities.
- `TetherChainActive` — resource inserted when chain mode is active.
- `maintain_tether_chain` — FixedUpdate system (.run_if(resource_exists::<TetherChainActive>)).before(tick_tether_beam)).

**fire/reverse split is now 4 methods each** (not 3 fire / 2 reverse):
- fire: `fire` → `fire_aoe_and_spawn` → `fire_utility_and_spawn` → `fire_breaker_effects`
- reverse: `reverse` → `reverse_aoe_and_spawn` → `reverse_utility` → `reverse_breaker_effects`

**Design docs already existed and are correct:**
- `docs/design/effects/tether_beam.md` — documents chain: bool field correctly (standard/chain mode sections).
- `docs/design/effects/flash_step.md` — correct.
- `docs/design/effects/mirror_protocol.md` — correct.
- `docs/design/effects/anchor.md` — correct.
- `docs/design/effects/circuit_breaker.md` — correct.
- `docs/design/effects/spawn_bolts.md` — inherit field already documented.

**New layout modules confirmed as directory modules:**
- `anchor/`, `circuit_breaker/`, `mirror_protocol/` — directory modules.
- `flash_step.rs` — single file.

## Confirmed Correct / Fixed (Effective* cache removal, feature/scenario-coverage, 2026-03-30)

- All 6 `Effective*` components removed: `EffectiveDamageMultiplier`, `EffectiveSpeedMultiplier`, `EffectiveSizeMultiplier`, `EffectivePiercing`, `EffectiveBumpForce`, `EffectiveQuickStop`.
- `EffectSystems::Recalculate` set removed from `effect/sets.rs` (only `Bridge` remains).
- `recalculate_*` systems removed from all effect modules; `register()` may be empty or wire only non-recalculate systems.
- `SizeBoostInRange` and `InjectWrongSizeMultiplier` invariant/mutation removed from scenario runner.
- `docs/architecture/data.md` — "Active/Effective Component Pattern" section rewritten to "Active Component Pattern" (direct-read model).
- `docs/architecture/plugins.md` — Effect File Pattern code snippet updated: removed `recalculate_speed` from `register()`, added `_source_chip: &str` to `fire()`/`reverse()` signatures, added `.multiplier()` method.
- `docs/architecture/effects/core_types.md` — Per-Effect Modules section: `app.add_systems(FixedUpdate, recalculate_speed)` in register() body replaced with a comment explaining simple stat effects have no runtime systems.
- `docs/architecture/ordering.md` and `docs/architecture/plugins.md` — `EffectSystems::Recalculate` already removed from Defined Sets table by team before this session.
- `docs/architecture/standards.md` — already correct: 23 invariants, `SizeBoostInRange` not in list.

## RON Format Confirmed (2026-03-30)

- Chip template fields: `common:`, `uncommon:`, `rare:`, `legendary:` — NOT `Some((...))`; absence means the slot is not present
- Effect dispatch in RON: `On(target: Bolt, then: [Do(Piercing(1))])` — top-level wrapper is `RootEffect::On`, not `When(trigger: OnSelected, ...)`
- Trigger chip names: `DamageBoost(1.1)` not `DamageBoost(0.1)` for the rare Piercing chip example

## Confirmed Correct / Fixed (bolt builder migration, feature/chip-evolution-ecosystem, 2026-03-31)

**What the builder migration changed (relative to prior "Current State" docs):**
- `init_bolt_params` DELETED. `Bolt::builder()` in `bolt/builder.rs` inserts all components at spawn time.
- `prepare_bolt_velocity` DELETED. `apply_velocity_formula` at each collision/steering site. No separate step.
- `BoltSystems::InitParams` DOES NOT EXIST. Only: `Reset`, `CellCollision`, `WallCollision`, `BreakerCollision`, `BoltLost`.
- `spawn_extra_bolt` free function REMOVED from `effect/effects/fire_helpers.rs`. Each effect module calls `Bolt::builder()` directly.
- `MaxReflectionAngle` RENAMED to `BreakerReflectionSpread` in `breaker/components/core.rs`. Config field is `reflection_spread` (degrees), converted via `.to_radians()`.
- `PrimaryBolt` — new marker component on baseline bolt entity (builder `.primary()`). Separate from `BoltServing`.

**Docs updated 2026-03-31:**
- `docs/architecture/data.md` — `MaxReflectionAngle` → `BreakerReflectionSpread` (x3); `Without<BreakerMaxSpeed>` → `Without<MaxSpeed>`
- `docs/architecture/layout.md` — `effects/mod.rs` description: removed `spawn_extra_bolt helper`
- `docs/architecture/plugins.md` — `effects/mod.rs` description: removed `spawn_extra_bolt helper`
- `docs/architecture/effects/structure.md` — `effects/mod.rs` line: removed `spawn_extra_bolt helper`
- `docs/architecture/bolt-definitions.md` — "Current State" section updated: spawn flow (builder, no init_bolt_params), extra bolt spawn (direct builder calls), bolt-lost (component reads), breaker→bolt relationship (angle constraints).

**Intentionally forward-looking in bolt-definitions.md (do NOT flag as drift):**
- Target State / BoltRenderingConfig, AttachVisuals, sync_bolt_visual_modifiers — not yet implemented (requires rantzsoft_vfx)
- Dynamic Visual Modifier System section — future phase

**State after Waves 1-8 (feature/breaker-builder-pattern, 2026-04-01):**
- `BoltConfig` ELIMINATED. `BoltRegistry` + `BoltDefinition` are the production types.
- `BoltRespawnOffsetY`, `BoltRespawnAngleSpread`, `BoltInitialAngle` ELIMINATED. Replaced by `BoltSpawnOffsetY` + `BoltAngleSpread` (single component each, initialized from constants).
- `defaults.bolt.ron` deleted. `assets/bolts/default.bolt.ron` is the bolt definition RON.
- `BoltRadius` is now a type alias for `BaseRadius` from `shared/size.rs`.
- `BreakerConfig`, `BreakerStatOverrides`, `BreakerDefaults` ELIMINATED. `BreakerDefinition` has all 36+ fields.
- Registry extensions: `BreakerRegistry` uses `&["breaker.ron"]` (NOT `bdef.ron`).
- Builder: `BreakerBuilder<D, Mv, Da, Sp, Bm, V, R>` has 7 dimensions including Role.
- `.definition()` replaces `.config()` on both bolt and breaker builders.
- `spawn_or_reuse_breaker` replaces 4 init systems (`spawn_breaker`, `init_breaker_params`, `init_breaker`, `dispatch_breaker_effects`).
- `BreakerSystems::InitParams` variant does NOT exist.
- Component renames: `BreakerVelocity` → `Velocity2D`, `BreakerState` → `DashState`, `BreakerWidth/Height` → `BaseWidth/BaseHeight`, `EntityScale` → `NodeScalingFactor`, `BumpVisualParams` → `BumpFeedback`, `BumpVisual` → `BumpFeedbackState`.

## Confirmed Correct / Fixed (steering model + gravity_well split, feature/chip-evolution-ecosystem, 2026-04-01)

**gravity_well is now a directory module:**
- `breaker-game/src/effect/effects/gravity_well/` — directory with `mod.rs`, `effect.rs`, `tests/` subdirectory.
- `apply_gravity_pull` lives at `gravity_well/effect.rs`.
- `layout.md` and `plugins.md` updated: `gravity_well.rs` → `gravity_well/` directory module.

**speed_boost::fire() / reverse() now call recalculate_velocity:**
- After pushing/removing the multiplier, `recalculate_velocity(entity, world)` is called to invoke `apply_velocity_formula` immediately.
- This is the third Velocity2D write path in the effect domain (alongside gravity_well and attraction).
- `plugins.md` Velocity2D exception section updated to document the third path.
- `plugins.md` Effect File Pattern and `core_types.md` Per-Effect Modules code comments updated to mention recalculate_velocity.

**InvariantKind — BoltSpeedInRange renamed to BoltSpeedAccurate:**
- Code: `BoltSpeedAccurate` (not `BoltSpeedInRange`).
- `standards.md` updated: both invariant list and scenario runner invariant list now say `BoltSpeedAccurate`.
- Total: still 23 variants.

**MutationKind — two variants removed (Effective* cache removal):**
- `InjectWrongSizeMultiplier` and `InjectWrongEffectiveSpeed` no longer exist in code.
- Total MutationKind variants: 16 (was 18 in stale docs; was 18 in previous known-state memory — SUPERSEDED).
- `docs/design/terminology/scenarios.md` updated: removed those two variants from the list.

**Rendering docs (docs/architecture/rendering/) are ALL forward-looking Phase 5 design docs:**
- `module-map.md`, `scheduling.md`, `materials.md`, `error-handling.md`, `screen-migration.md`, `chip_cards.md`, `hud.md`, `screen_effects.md`, `communication.md`, etc.
- `rantzsoft_vfx` crate does NOT YET EXIST in code. Do NOT flag as drift.
- Phase 5 plan docs (5d, 5r, 5u, 5w, etc.) are also all planning docs, not yet implemented.

## Confirmed Correct / Fixed (breaker-builder-pattern Wave 9, feature/breaker-builder-pattern, 2026-04-02)

**`TriggerContext` added to `transfer_effect` signature:**
- `EffectCommandsExt::transfer_effect` now takes 5 params: `(entity, chip_name, children, permanent, context: TriggerContext)`.
- `TriggerContext { bolt, breaker, cell, wall: Option<Entity> }` — bridge systems populate fields they know about.
- Fixed in `docs/architecture/messages.md` and `docs/architecture/plugins.md`.

**Bolt builder `spawn()` takes `&mut World`, not `&mut Commands`:**
- Bolt builder's `spawn()` signature is `fn spawn(self, world: &mut World) -> Entity`.
- Breaker builder's `spawn()` still takes `&mut Commands` — the two builders differ.
- `spawn_bolt` system signature is `fn spawn_bolt(world: &mut World)` (exclusive world access).
- Fixed in `docs/architecture/builders/bolt.md` (spawn() Behavior section) and `docs/architecture/builders/pattern.md`.

**`dispatch_bolt_effects` runs in FixedUpdate, NOT OnEnter:**
- System runs in FixedUpdate with `Added<BoltDefinitionRef>` filter, `.before(EffectSystems::Bridge)`.
- Effects dispatch on first FixedUpdate tick after bolt spawns (not synchronously in OnEnter).
- `docs/architecture/ordering.md` updated: added to FixedUpdate chain; noted in OnEnter description.

**`spawn_lives_display` does NOT exist in code:**
- Was listed in ordering.md's OnEnter chain — removed.
- `LivesCount` is a component on the breaker entity (not a UI system in effect domain).

**Breaker QueryData struct names:**
- `BreakerBumpData` does not exist. Actual structs: `BreakerBumpTimingData`, `BreakerBumpGradingData`, `SyncBreakerScaleData`.
- Fixed in `docs/architecture/builders/breaker.md` Key Files list.

**Ordering chain additions:**
- `apply_node_scale_to_bolt` added to OnEnter chain.
- `cleanup_destroyed_bolts.after(EffectSystems::Bridge)` added to FixedUpdate chain.
- `bolt_cell_collision` now shows `.after(EnforceDistanceConstraints)` in the MaintainQuadtree section.

**`BreakerState` renamed to `DashState`; `BoltSpeed` never existed:**
- `docs/design/terminology/core.md` code examples updated.

**`bolt-definitions.md` Target State / planning sections (do NOT flag as drift):**
- All sections 4-16 under "System Changes Required" are planning docs for future bolt visuals phase.
- `BoltRenderingConfig`, `AttachVisuals`, `sync_bolt_visual_modifiers` — not yet implemented.
- Pseudo-code uses `BoltRadius(def.radius)` as shorthand even though it's a type alias — intentional planning notation.

## State Folder Restructure + crate-routing-migration Drift — Partially Fixed (2026-04-03)

**Wave 8 of state-lifecycle-refactor is still pending. Only the most egregious doc drift has been fixed.**

**Fixed on 2026-04-03 (refactor/crate-routing-migration context):**
- `docs/architecture/state.md` — COMPLETELY REWRITTEN. Now describes the 4-level hierarchy (AppState → GameState → MenuState/RunState → NodeState/ChipSelectState/RunEndState), `rantzsoft_lifecycle` routing, declarative `Route` API, `ChangeState<S>` messages, transition effects, pause model, and `CleanupOnExit<S>`.
- `docs/architecture/ordering.md` — Section header `OnEnter(GameState::Playing)` → `OnEnter(NodeState::Loading)`; domain labels updated (`[ui domain]` → `[state/run/node/hud domain]`, `[wall domain]` → `[walls domain]`, `[run/node domain]` → `[state/run/node domain]`); `UiSystems::SpawnTimerHud` and `NodeSystems` source paths updated from `ui/sets.rs` / `run/node/sets.rs` to correct paths.
- `docs/architecture/ordering.md` — Section `OnEnter(GameState::TransitionOut) / OnEnter(GameState::TransitionIn)` REMOVED and replaced with "Transition Lifecycle (rantzsoft_lifecycle)" note explaining those states no longer exist.
- `docs/architecture/ordering.md` — Section `OnExit(GameState::MainMenu)` renamed to `OnExit(MenuState::Main)`.
- `docs/architecture/messages.md` — `WallsSpawned` sender updated: `wall (spawn_walls)` → `walls (state/run/node/systems/spawn_walls)`.
- `docs/architecture/messages.md` — `ChipSelected` sender updated: `UI (handle_chip_input)` → `state/run/chip_select (handle_chip_input)`.

**Still deferred to Wave 8 (do NOT flag again until after Wave 8 merges):**
- `docs/architecture/plugins.md` — Domain Layout table still shows `screen/`, `ui/`, `run/`, `wall/`; Plugin registration order still lists `ScreenPlugin`, `UiPlugin`, `RunPlugin` (not `StatePlugin`); Scenario runner exception paragraph still mentions `screen` and `wall` module paths; `UiSystems` location cited as `ui/sets.rs`; `rantzsoft_lifecycle` not listed in workspace layout table.
- `docs/architecture/data.md` — WallRegistry "Re-exported from `wall/`" (now `walls/`)
- `docs/architecture/builders/pattern.md` — Wall builder location shown as `breaker-game/src/wall/builder/` (now `breaker-game/src/walls/builder/`)

## Confirmed Correct / Fixed (wall-builder-pattern feature, 2026-04-02)

**Wall builder fully implemented:**
- `Wall::builder()` in `walls/builder/` with `WallBuilder<S, V>` — 2 generic params (Side, Visual). (domain renamed from `wall/` to `walls/` in state lifecycle refactor)
- Side dimension: `NoSide` → `Left` / `Right` / `Ceiling` / `Floor` (required, transitions at `.left()` / `.right()` / `.ceiling()` / `.floor()`).
- Visual dimension: `Invisible` (default) / `Visible` (`.visible(meshes, materials)`) — not a typestate gate, both Invisible and Visible have `build()` + `spawn()`.
- Lifetime: stored enum `Permanent` / `Timed(f32)` / `OneShot`, Floor-only setters `.timed()` / `.one_shot()`. NOT a generic dimension.
- `build()` returns `impl Bundle` for both Invisible and Visible paths. `spawn()` takes `&mut Commands`.
- Effects dispatched inline in `spawn()` via `push_bound_effects` — no separate dispatch system.
- `WallDefinition` — `Asset + TypePath + Deserialize + Clone + Debug` with fields: `name`, `half_thickness` (default 90.0), `color_rgb: Option<[f32; 3]>` (default None), `effects: Vec<RootEffect>` (default empty).
- `WallRegistry` — `Resource`, implements `SeedableRegistry`. `asset_dir() = "walls"`, `extensions() = ["wall.ron"]`.
- `WallSize` component — **DELETED**. Walls use `Scale2D` + `Aabb2D` from builder geometry.
- `dispatch_wall_effects` system — **DELETED**. Effect dispatch is inline in `spawn()`.
- `spawn_walls` migrated: reads `WallRegistry`, calls `Wall::builder()` three times (left, right, ceiling).
- `docs/todos/TODO.md` item 2 marked `[done]`.
- `docs/architecture/ordering.md` OnEnter chain updated: `(spawn_walls, dispatch_wall_effects).chain()` → `spawn_walls` alone.
- `docs/architecture/builders/pattern.md` Current Implementations table: Wall row added.
- `docs/architecture/data.md` Key Types registry table: WallRegistry row added. Rule 2 example updated.

**Intentionally forward-looking (do NOT flag as drift):**
- `Lifetime::Timed(f32)` and `Lifetime::OneShot` variants exist in builder code but are `allow(dead_code)` for non-Floor walls — they're future API for Shield/SecondWind chip floor walls (Phase 5j).
- `visible()` transition and `WallBuilder<S, Visible>` exist but are `allow(dead_code)` — future API for Phase 5j visual walls.
- `with_half_thickness()`, `with_color()`, `with_effects()`, `invisible()` builder methods are `allow(dead_code)` — future API.

## Confirmed Correct / Fixed (Shield refactor, develop branch, 2026-04-02)

**Shield effect completely rewritten — ShieldActive eliminated:**
- `ShieldActive` component NO LONGER EXISTS anywhere in production code. Do NOT flag its absence.
- `EffectKind::Shield { duration: f32 }` — field changed from `stacks: u32` to `duration: f32`.
- `ShieldWall` — new marker component on the floor wall entity spawned by Shield effect.
- `ShieldWallTimer(Timer)` — new timer component; `tick_shield_wall_timer` despawns wall when expired.
- `fire()` spawns a visible blue HDR floor wall (Wall + ShieldWall + ShieldWallTimer + Mesh2d + MeshMaterial2d). If wall exists, resets timer in-place.
- `reverse()` despawns all ShieldWall entities.
- `parry.chip.ron` now uses `Shield(duration: 5.0)` (was `Shield(stacks: 1)`).
- Invariant `ShieldChargesConsistent` RENAMED to `ShieldWallAtMostOne` (checks count <= 1, not charge consistency).
- The "ShieldActive Cross-Domain Write Exception" section in plugins.md was deleted — no longer applies.

**Intentionally forward-looking in graphics catalog docs (do NOT flag as drift):**
- `docs/design/graphics/catalog/entities.md` — "Bolt shield aura" row and "Shield barrier" implementation text reference `ShieldActive`. These are future Phase 5 rendering design docs describing planned VFX that will be redesigned for the new wall-based Shield when Phase 5 arrives. Do not edit — they are planning artifacts.
- `docs/design/graphics/catalog/effects.md` — "Shield (bolt-loss)" row and implementation text reference `ShieldActive`. Same reasoning — future rendering design.
- `docs/todos/detail/rendering-refactor/walls_and_background.md` — "Shield Barrier" section references `ShieldActive`. Planning doc for future rendering work.
- `docs/todos/detail/rendering-refactor/communication.md` — Shield VFX section references `ShieldActive`. Planning doc.
- `docs/todos/detail/game-crate-splitting/research/cross-domain-dependencies.md` — references `ShieldActive`. Historical research artifact; reflects state at time of research.
