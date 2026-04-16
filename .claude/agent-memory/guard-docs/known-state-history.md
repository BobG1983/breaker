---
name: known-state-history
description: Confirmed doc/code alignment from older sessions (2026-03-28 through 2026-04-06). Current state is in known-state.md.
type: project
---

## Confirmed Correct (as of pause-quit cleanup, feature/effect-placeholder-visuals, 2026-04-06)

**CleanupOnNodeExit / CleanupOnRunEnd FULLY REMOVED — migration complete:**
- `CleanupOnNodeExit` and `CleanupOnRunEnd` DO NOT EXIST in `breaker-game/src/`. Removed.
- All entity lifecycle markers are now `CleanupOnExit<NodeState>` and `CleanupOnExit<RunState>` from `rantzsoft_stateflow`.
- `shared/components.rs` no longer contains any cleanup markers — it only has `BaseWidth`, `BaseHeight`, `NodeScalingFactor`.

**NodeResult::Quit added:**
- `NodeResult` enum now has 5 variants: `InProgress`, `Won`, `TimerExpired`, `LivesDepleted`, `Quit`.
- `NodeResult::Quit` routes to `RunState::Teardown` via `resolve_node_next_state()` in `state/plugin.rs`. Skips `RunEnd` screen.

**Bolt builder spawn() takes &mut Commands (NOT &mut World):**
- All terminal `spawn()` impls in `bolt/builder/core/terminal.rs` take `&mut Commands`.
- Effect modules inside `fire()` (which takes `&mut World`) bridge via `CommandQueue`.
- The 2026-04-02 memory entry "Bolt builder `spawn()` takes `&mut World`" is SUPERSEDED.

**Intentionally NOT edited (historical research artifacts — do NOT flag as drift):**
- `docs/todos/detail/cross-domain-prelude/research-*.md`, `docs/todos/detail/game-crate-splitting/research/*.md`, `docs/todos/detail/wall-builder-pattern/research/*.md`, `docs/todos/detail/killed-trigger-damage-attribution/research/*.md`, `docs/todos/detail/effect-desugaring-node-running-trigger/research/*.md` — all historical snapshots. Do not edit.
- `docs/architecture/bolt-definitions.md` "Target State" pseudo-code uses `CleanupOnRunEnd` — forward-looking planning code, clearly labeled. Do NOT edit.

---

## Confirmed Correct / Fixed (Shield refactor, develop branch, 2026-04-02)

- `ShieldActive` component NO LONGER EXISTS anywhere in production code. Do NOT flag its absence.
- `EffectType::Shield(ShieldConfig { duration: f32 })` — field changed from `stacks: u32` to `duration: f32`.
- `ShieldWall` + `ShieldWallTimer(Timer)` — new components. `tick_shield_wall_timer` despawns wall when expired.
- `parry.chip.ron` now uses `Shield(duration: 5.0)` (was `Shield(stacks: 1)`).
- Invariant `ShieldChargesConsistent` RENAMED to `ShieldWallAtMostOne`.
- "ShieldActive Cross-Domain Write Exception" section in plugins.md was deleted.

**Intentionally forward-looking (do NOT flag as drift):**
- `docs/design/graphics/catalog/entities.md`, `effects.md`, `docs/todos/detail/rendering-refactor/*.md` — reference `ShieldActive` in future VFX planning docs. Do not edit.
- `docs/todos/detail/game-crate-splitting/research/cross-domain-dependencies.md` — historical research artifact.

---

## Confirmed Correct / Fixed (wall-builder-pattern feature, 2026-04-02)

- `Wall::builder()` in `walls/builder/` with `WallBuilder<S, V>` — 2 generic params (Side, Visual).
- `WallSize` component — **DELETED**. Walls use `Scale2D` + `Aabb2D` from builder geometry.
- `dispatch_wall_effects` system — **DELETED**. Effect dispatch is inline in `spawn()`.
- `spawn_walls` migrated: reads `WallRegistry`, calls `Wall::builder()` three times (left, right, ceiling).
- `WallDefinition`: `name`, `half_thickness` (default 90.0), `color_rgb: Option<[f32; 3]>`, `effects: Vec<RootNode>`.
- `WallRegistry` — `Resource`, implements `SeedableRegistry`. `asset_dir() = "walls"`, `extensions() = ["wall.ron"]`.

**Intentionally forward-looking (do NOT flag as drift):**
- `Lifetime::Timed(f32)` and `Lifetime::OneShot` variants exist but are `allow(dead_code)` — future Phase 5j API.
- `visible()` transition and `WallBuilder<S, Visible>` exist but are `allow(dead_code)` — future API.

---

## Confirmed Correct / Fixed (breaker-builder-pattern Wave 9, feature/breaker-builder-pattern, 2026-04-02)

- `EffectCommandsExt` methods at the time: `transfer_effect` with 5 params. NOTE: `transfer_effect` was removed in effect_v3 refactor; replaced by `stamp_effect` / `route_effect`.
- `spawn_or_reuse_breaker` replaces 4 init systems (`spawn_breaker`, `init_breaker_params`, `init_breaker`, `dispatch_breaker_effects`).
- `BreakerSystems::InitParams` variant does NOT exist.
- Component renames: `BreakerVelocity` → `Velocity2D`, `BreakerState` → `DashState`, `BreakerWidth/Height` → `BaseWidth/BaseHeight`, `EntityScale` → `NodeScalingFactor`, `BumpVisualParams` → `BumpFeedback`, `BumpVisual` → `BumpFeedbackState`.
- `dispatch_bolt_effects` runs in FixedUpdate with `Added<BoltDefinitionRef>` filter, `.before(EffectSystems::Bridge)`.
- `BreakerBumpData` does not exist. Actual structs: `BreakerBumpTimingData`, `BreakerBumpGradingData`, `SyncBreakerScaleData`.

---

## State Folder Restructure + crate-routing-migration Drift — Partially Fixed (2026-04-03)

**Wave 8 of state-lifecycle-refactor was still pending as of 2026-04-03. Only the most egregious doc drift was fixed.**

**Fixed on 2026-04-03:**
- `docs/architecture/state.md` — COMPLETELY REWRITTEN. Now describes 4-level hierarchy with `rantzsoft_stateflow`, declarative `Route` API, `ChangeState<S>` messages, transition effects, pause model, and `CleanupOnExit<S>`.
- `docs/architecture/ordering.md` — Section header `OnEnter(GameState::Playing)` → `OnEnter(NodeState::Loading)`.
- `docs/architecture/ordering.md` — Section `OnEnter(GameState::TransitionOut)/TransitionIn` REMOVED; replaced with "Transition Lifecycle (rantzsoft_stateflow)" note.
- `docs/architecture/messages.md` — `WallsSpawned` and `ChipSelected` senders updated to current paths.

**Still deferred to Wave 8 (do NOT flag again until after Wave 8 merges):**
- `docs/architecture/plugins.md` — Domain Layout table still shows `screen/`, `ui/`, `run/`, `wall/`; Plugin registration order still lists `ScreenPlugin`, `UiPlugin`, `RunPlugin`.
- `docs/architecture/data.md` — WallRegistry "Re-exported from `wall/`" (now `walls/`)
- `docs/architecture/builders/pattern.md` — Wall builder location shown as `breaker-game/src/wall/builder/` (now `breaker-game/src/walls/builder/`)

---

## Confirmed Correct / Fixed (steering model + gravity_well split, feature/chip-evolution-ecosystem, 2026-04-01)

- `gravity_well` is now a directory module: `breaker-game/src/effect/effects/gravity_well/`.
- `speed_boost::fire()` / `reverse()` now call `recalculate_velocity(entity, world)` after pushing/removing the multiplier.
- `BoltSpeedInRange` RENAMED to `BoltSpeedAccurate`. `standards.md` updated.
- `InjectWrongSizeMultiplier` and `InjectWrongEffectiveSpeed` no longer exist. MutationKind total: 16.
- `docs/architecture/rendering/` files are ALL forward-looking Phase 5 design docs. `rantzsoft_vfx` crate does NOT YET EXIST.

---

## Architecture Confirmed (source-chip-shield-absorption, 2026-03-29)

- Effect dispatch: `fire_dispatch(effect: &EffectType, entity, source: &str, world)` / `reverse_dispatch(effect: &ReversibleEffectType, entity, source: &str, world)` — free functions, source_chip on ALL signatures.
- `EffectCommandsExt` methods: `fire_effect(entity, EffectType, String)`, `reverse_effect(entity, ReversibleEffectType, String)`, `route_effect(entity, String, Tree, RouteType)`, `stamp_effect(entity, String, Tree)`, `stage_effect(entity, String, Tree)`, `remove_effect(entity, &str)`, `remove_staged_effect(entity, String, Tree)`, `track_armed_fire(owner, String, Entity)`.
- `CellEffectsDispatched` — marker component in `cells/components/types.rs`; prevents double-dispatch.
- `dispatch_cell_effects` — cells system; `OnEnter(GameState::Playing)` after `NodeSystems::Spawn`.
- `dispatch_breaker_effects` — **SUPERSEDED** by `spawn_or_reuse_breaker` in feature/breaker-builder-pattern.
- `dispatch_wall_effects` — **DELETED** in wall-builder-pattern feature.
- `ChainArcCountReasonable` — new `InvariantKind` variant.
- InvariantKind total: 22 variants (verified 2026-04-06). `BoltSpeedInRange` renamed to `BoltSpeedAccurate`.
- MutationKind total: 17 variants (updated 2026-04-06). First variant is `SetDashState`.
- `EffectType::Shield(ShieldConfig { duration: f32 })` — timed visible floor wall (`ShieldWall` + `ShieldWallTimer`).
- Stat model (AFTER effect_v3 refactor): `EffectStack<T>` stacks → consumers call `.multiplier()` / `.total()` / `.aggregate()` directly. NO `Effective*` components. `EffectV3Systems` has `Bridge`, `Tick`, `Conditions`, `Reset`.
- Dispatch: `fire_dispatch()` and `reverse_dispatch()` are free functions in `effect_v3/dispatch/`.

## RON Format Confirmed (2026-03-30)

- Chip template fields: `common:`, `uncommon:`, `rare:`, `legendary:` — NOT `Some((...))`; absence means slot not present.
- Effect dispatch in RON: `Stamp(Bolt, Fire(Piercing(PiercingConfig(count: 1))))` — top-level wrapper is `RootNode::Stamp`.

---

## Confirmed Correct (bolt builder migration, feature/chip-evolution-ecosystem, 2026-03-31)

- `init_bolt_params` DELETED. `prepare_bolt_velocity` DELETED. `BoltSystems::InitParams` DOES NOT EXIST.
- `spawn_extra_bolt` free function REMOVED from `effect/effects/fire_helpers.rs`.
- `MaxReflectionAngle` RENAMED to `BreakerReflectionSpread` in `breaker/components/core.rs`.
- `PrimaryBolt` — new marker component on baseline bolt entity (builder `.primary()`).
- `BoltConfig` ELIMINATED. `BoltRegistry` + `BoltDefinition` are the production types.
- `BoltRespawnOffsetY`, `BoltRespawnAngleSpread`, `BoltInitialAngle` ELIMINATED.
- `defaults.bolt.ron` deleted. `assets/bolts/default.bolt.ron` is the bolt definition RON.
- `BoltRadius` is now a type alias for `BaseRadius` from `shared/size.rs`.

---

## Confirmed Correct (file-split refactor, 2026-03-30)

- `effect/core/types.rs` was split to `effect/core/types/` directory module (old system). Now superseded by `effect_v3/types/` (flat dir, one file per type).
- `effect/core/types/definitions.rs` was split to `effect/core/types/definitions/` (old system). Now superseded by `effect_v3/types/`.
- Many effect modules are now directory modules (shockwave/, chain_bolt/, chain_lightning/, explode/, tether_beam/, pulse/, piercing_beam/, attraction/, spawn_bolts/, spawn_phantom/, entropy_engine/, second_wind/, random_effect/).
- Trigger modules evaluate/, impact/, impacted/, until/ are now directory modules.
- `EvolutionRegistry` → `EvolutionTemplateRegistry` in plugins.md and plan/index.md.
- `ChipRegistry` → `ChipTemplateRegistry`/`ChipCatalog` in content.md registries section.
- `EffectChains` references in evolutions.md replaced with `BoundEffects(pub Vec<(String, Tree)>)`.

---

## Confirmed Correct (effect system rewrite, 2026-03-28)

- `docs/architecture/messages.md` — Collision messages use `BoltImpactCell`, `BoltImpactWall`, `BreakerImpactCell`, `BreakerImpactWall`, `CellImpactWall`. `DamageDealt<Cell>.source_chip` (not `source_bolt`; `DamageCell` replaced by unified `DamageDealt<T>`).
- `docs/architecture/effects/core_types.md` — `EffectType` enum includes `Explode`, `QuickStop`, `TetherBeam`. `SecondWind` wraps config struct. `EntropyEngine` uses `max_effects: u32`.
- `docs/architecture/messages.md` — SpawnAdditionalBolt REMOVED. Effects spawn directly via `&mut World`.
- `docs/plan/index.md` — Runtime Effects entry marked Done.
- `docs/architecture/effects/core_types.md` — EffectType enum is complete and current for all 25 effect modules.

## SpawnAdditionalBolt — REMOVED (feature/runtime-effects)

`SpawnAdditionalBolt` was removed from `bolt/messages.rs` and from `docs/architecture/messages.md`.
`spawn_bolts::fire()` and `chain_bolt::fire()` spawn directly via `&mut World`.
Do NOT flag its absence as missing functionality — direct World spawning is the established pattern.

## Key Architectural Fact: DamageDealt<Cell> pre-bakes multiplier

`apply_damage<Cell>` (unified death pipeline) does NOT read damage multipliers directly. `bolt_cell_collision`
reads `ActiveDamageBoosts` (NOT `EffectiveDamageMultiplier` — that type no longer exists) and calls
`.multiplier()` to compute `effective_damage = BASE_BOLT_DAMAGE * mult` — that pre-computed value
goes into the `DamageDealt<Cell>.damage` field. The old `DamageCell` type and `handle_cell_hit`
system are both gone; replaced by the unified `DamageDealt<T>` message and `apply_damage<T>` generic.
**Do not flag cells reading `Active*` types as missing — it's correct.**

## Intentionally Forward-Looking (do NOT flag as drift)

- `docs/design/chip-catalog.md` — Chip RON files now exist under `breaker-game/assets/chips/` (34+ templates). The doc's additive format vs RON multiplicative format divergence is a known blocker.
- Evolution chips (Entropy Engine, Nova Lance, etc.) — Not yet implemented in code. Design spec only.
- `docs/plan/index.md` — Spatial/Physics Extraction and Stat Effects are both correctly marked Done.

## chips/components.rs — Intentional Stub

The file contains only a doc comment. Chip stat components (DamageBoost, BoltSpeedBoost, etc.) were removed; state is now managed by effect domain `Active*` components.

## Confirmed Correct / Fixed (Effective* cache removal, feature/scenario-coverage, 2026-03-30)

- All 6 `Effective*` components removed: `EffectiveDamageMultiplier`, `EffectiveSpeedMultiplier`, `EffectiveSizeMultiplier`, `EffectivePiercing`, `EffectiveBumpForce`, `EffectiveQuickStop`.
- `EffectSystems::Recalculate` set removed from `effect/sets.rs` (only `Bridge` remains).
- `SizeBoostInRange` and `InjectWrongSizeMultiplier` invariant/mutation removed from scenario runner.
