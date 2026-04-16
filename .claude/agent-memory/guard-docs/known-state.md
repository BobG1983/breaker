---
name: known-state
description: Confirmed doc/code alignment state for current and recent sessions; older session history in known-state-history.md
type: project
---

## Confirmed Correct (as of test-infrastructure-consolidation, 2026-04-09)

- `docs/architecture/testing.md` — fully rewritten to match actual implementation (TestAppBuilder, MessageCollector<M>, tick, domain spawners)
- `docs/todos/TODO.md` — item 1 marked `[done]` (was `[ready]`, consolidation shipped in 195-file change)
- `shared/test_utils/` is a **directory module** (`builder.rs`, `collector.rs`, `tick_helper.rs`, `mod.rs`, `tests/`) — NOT a single file
- `TestAppBuilder<S>` is a typestate builder (`NoStates` → `WithStates`); no free-function `with_state_hierarchy(app)` or `enter_playing(app)` exist
- State navigation methods: `.in_state_node_playing()` and `.in_state_chip_selecting()` (not `enter_playing`)
- `MessageCollector<M>` is generic — no per-message collector structs; registered via `.with_message_capture::<M>()`
- Domain `test_utils` contain spawners/definitions ONLY — no app builders (bolt, breaker, cells, walls have test_utils; effect/chips/state do not)
- `spawn_in_world()` helper is ELIMINATED (47→0), not consolidated — Bevy 0.18 `World::commands()` + `World::flush()` native

## Confirmed Correct (as of toughness-hp-scaling, 2026-04-08)

- `docs/design/decisions/node-type-differentiation.md` — HP Scaling section updated: removed stale `TierDefinition.hp_mult` claim; now describes `Toughness` enum + `ToughnessConfig` model
- `docs/design/terminology/run.md` — Tier row: removed "HP multiplier" from parameter list; added `Toughness` and `ToughnessConfig` as new glossary entries
- `NodeAssignment` fields confirmed: `node_type`, `tier_index`, `timer_mult` — NO `hp_mult`
- `TierDefinition` fields confirmed: `nodes`, `active_ratio`, `timer_mult`, `introduced_cells` — NO `hp_mult`
- `DifficultyCurve` fields confirmed: `tiers`, `timer_reduction_per_boss` — NO `boss_hp_mult`
- `NodeOutcome` fields confirmed: `node_index`, `result`, `cleared_this_frame` (NOT `transition_queued`), `tier`, `position_in_tier`
- `ToughnessConfig` fields confirmed: `weak_base`, `standard_base`, `tough_base`, `tier_multiplier`, `node_multiplier`, `boss_multiplier`
- Research snapshots in `docs/todos/detail/mod-system-design/research/` (run-state-flow.md, tier-stub-trace.md, message-component-patterns.md, chip-offering-flow.md) are intentionally historical — written before toughness landed, DO NOT flag their `hp_mult` / `transition_queued` references as drift

### Key facts for toughness-hp-scaling

- `Toughness` enum: `Weak` (default `Standard`), `Standard`, `Tough` — lives in `cells/definition/data.rs`
- `ToughnessConfig` resource: lives in `cells/resources/data.rs`, loaded from `defaults.toughness.ron`
- `boss_multiplier` (was `boss_hp_mult`) is now on `ToughnessConfig`, not on `DifficultyCurve` or `TierDefinition`
- `cleared_this_frame` (was `transition_queued`) is the tie-frame guard on `NodeOutcome`
- `stubbing-tiers.md` spec is STALE: proposes adding `current_tier` to `NodeOutcome`, but `tier` and `position_in_tier` were already added by the toughness feature. Also still shows `transition_queued` field name. Needs human decision on whether to update or archive.

---

## Confirmed Correct (as of cell-builder-pattern, 2026-04-08)

- `docs/todos/TODO.md` — item 1 changed from `[in-progress]` (labelled "shielded") to `[done]` (corrected to "guarded")
- `docs/todos/DONE.md` — cell builder pattern entry added
- `docs/architecture/builders/cell.md` — fixed: `.hp()` marked as production transition (it's test-only); `.override_hp()` clarified to require HasHealth; `collect_guardian_slots()` reference removed (function does not exist); `typestate_tests.rs` added to file layout; `.rendered()` guardian pre-computation note added
- `docs/architecture/builders/pattern.md` — Cell builder row added to the Current Implementations table
- `docs/architecture/data.md` — `CellTypeRegistry` table row corrected: key is `String` not `char`; field is `behaviors: Option<Vec<CellBehavior>>` not `behavior: CellBehavior`; `SeedableRegistry` noted
- `docs/design/terminology/core.md` — added `GuardedCell`, `GuardianCell`, `LockCell`, `CellBehavior` entries

### Key facts for cell-builder-pattern

- `Cell::builder()` returns `CellBuilder<NoPosition, NoDimensions, NoHealth, Unvisual>`
- `.hp()` is `#[cfg(test)]` only — production MUST use `.definition(&def)` to set Health
- `.override_hp()` is on `impl<P,D,V> CellBuilder<P,D,HasHealth,V>` — requires HasHealth, not any typestate
- `GuardianSpawnConfig` fields: `hp`, `color_rgb`, `slide_speed`, `cell_height`, `step_x`, `step_y` — no dimensions field
- `collect_guardian_slots()` does NOT exist — slots are passed by callers from node layout data
- Test files in `cells/builder/tests/`: `typestate_tests.rs`, `build_tests.rs`, `definition_tests.rs`, `spawn_tests.rs`, `optional_tests.rs`, `integration_tests.rs`
- `CellTypeRegistry` keys by `String` alias (multi-char supported), implements `SeedableRegistry`, folder `assets/cells/`
- `NodeLayout.locks: Option<LockMap>` where `LockMap = HashMap<(usize,usize), Vec<(usize,usize)>>`
- `Headless` visual marker is `#[cfg(test)]` only — gated out of production builds
- Guardian initial `SlideTarget` set to `(slot + 1) % 8` at spawn time in `spawn_guardian_children`
- `slide_guardian_cells` system registered in `FixedUpdate` with `.run_if(in_state(NodeState::Playing))`

---

## Confirmed Correct (as of bolt-birthing-animation, 2026-04-08)

- `docs/todos/TODO.md` — item 2 changed from `[in-progress]` to `[done]`
- `docs/todos/DONE.md` — bolt birthing animation entry added with quit teardown chain note
- `docs/architecture/state.md` — `GameState::Teardown` and `AppState::Teardown` annotations corrected (both used by quit path); `TransitionType::None` + `with_dynamic_transition` documented; **Quit Teardown Chain** section added above Pause section
- `docs/architecture/ordering.md` — `OnEnter(NodeState::AnimateIn)` section added (`begin_node_birthing`); `tick_birthing` added to FixedUpdate section
- `docs/design/terminology/core.md` — `Birthing` entry added

### Key facts for this feature

- `Birthing` component lives in `shared/birthing.rs` (not a bolt-specific file); `BIRTHING_DURATION = 0.3s`
- `begin_node_birthing` runs `OnEnter(NodeState::AnimateIn)`, queries `(With<Bolt>, Without<Birthing>)`
- `tick_birthing` runs in `FixedUpdate` with `run_if(in_state(NodeState::AnimateIn).or(in_state(NodeState::Playing)))`
- Builder `.birthed()` method sets `optional.birthed = true`; the system (not the builder) handles spawning with zero scale
- Quit path: `MenuItem::Quit` → `MenuState::Teardown` (TransitionType::None) → `GameState::Teardown` (TransitionType::None, condition route) → `AppState::Teardown` (condition route) → `send_app_exit`
- `TransitionType::None` variant added to `rantzsoft_stateflow::TransitionType` enum

---

## Confirmed Correct (as of scenario-runner-wiring, 2026-04-07)

- `docs/architecture/standards.md` — Scenario Runner section: updated to reflect RunLog, output_dir, coverage, window tiling, screenshot-on-violation, fail-fast mode, `allowed_failures` field name, conditional invariant checkers (entered_playing gate), `--coverage` and `--clean` CLI flags
- `allowed_failures` — correct RON field name for self-test expected violations (was `expected_violations` in older docs, now removed)
- `ScenarioStats::entered_playing` — all invariant checkers gated on this flag; prevents false positives during loading
- `RunLog` — async mpsc + background thread, writes to `/tmp/breaker-scenario-runner/<date>/<N>.log`
- `StreamingPool` — count-based streaming pool in `streaming.rs`
- `tiling.rs` — pure grid math for parallel visual-mode window placement; `TilePosition`, env vars `SCENARIO_TILE_INDEX/COUNT` (`ENV_TILE_INDEX`/`ENV_TILE_COUNT`); functions `tile_config_env_vars`, `parse_tile_config`, `read_tile_config`, `grid_dimensions`, `tile_position`
- `coverage.rs` — `CoverageReport`, `check_coverage()`, `print_coverage_report()`; prints gaps only
- `discovery.rs` — RON parsed with `ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME)` (the "RON parse dedup" is really IMPLICIT_SOME extension)

## Intentionally forward-looking / do NOT flag

- `cargo.md` scenario runner options table — does not list `--fail-fast`, `--no-fail-fast`, `--clean`, `--coverage` flags. These were added on feature/scenario-runner-wiring but `.claude/rules/cargo.md` is not in guard-docs' edit scope. Needs human update.

---

## Confirmed Correct (as of salvo-wave-6a-6c, 2026-04-16)

- `docs/architecture/messages.md` — Added `SalvoImpactBreaker`, `PortalEntered`, `PortalCompleted`, `DamageDealt<Salvo>`, `KillYourself<T>` (added handle_portal_completed sender), `Destroyed<Salvo>`, `DespawnEntity` (added Salvo to sender list); also updated `DamageDealt<Cell>` senders to include tick_survival_timer and salvo_cell_collision
- `docs/architecture/ordering.md` — Added survival turret chain (tick_survival_timer → tick_salvo_fire_timer → fire_survival_turret), salvo collision systems (salvo_cell_collision, salvo_bolt_collision, salvo_breaker_collision, salvo_wall_collision), portal chain (check_portal_entry → handle_portal_entered → handle_portal_completed); updated prose reading section
- `docs/architecture/effects/collisions.md` — Added Salvo collision rows (Salvo↔Cell, Bolt, Breaker, Wall); replaced stale "Implementation Status" section (those systems are done) with "Salvo Collisions" section describing AABB-based detection behavior
- `docs/architecture/effects/death_pipeline.md` — Added Salvo to domain handlers table; fixed `DamageDealt` struct to include `source_chip` and `_marker`; fixed `KillYourself` and `Destroyed` structs to include `_marker`
- `docs/design/terminology/core.md` — Added `SurvivalTurret`, `Salvo`, `PortalCell` entries; updated `CellBehavior` entry to list all current variants (was only listing Regen/Guarded/Volatile)

### Key facts for salvo-wave-6a-6c

- `Salvo` struct is a marker component in `cells/behaviors/survival/salvo/components.rs`; carries `SalvoDamage(f32)`, `SalvoSource(Entity)`, `SalvoFireTimer(f32)` components
- `SALVO_FIRE_INTERVAL`, `SALVO_DAMAGE`, `SALVO_SPEED`, `SALVO_HALF_EXTENT` are constants in `salvo/components.rs`
- Salvo collision systems use AABB overlap (not quadtree): `salvo_cell_collision`, `salvo_bolt_collision`, `salvo_breaker_collision`, `salvo_wall_collision` — all in `cells/behaviors/survival/salvo/systems/`
- Salvo bypasses wall-entity collision; `salvo_wall_collision` checks against `PlayfieldConfig` bounds directly
- `Salvo` implements `GameEntity` and has full death pipeline monomorphization (apply_damage/detect_deaths/handle_kill registered in DeathPipelinePlugin) — but no current production sender of `DamageDealt<Salvo>` exists; salvo-on-bolt/breaker/wall collisions despawn via `commands.try_despawn()` directly
- `PortalEntered` has a `bolt` field gated behind `#[cfg(test)]` — production struct only has `portal: Entity`
- `handle_portal_entered` is a mock: immediately converts `PortalEntered` → `PortalCompleted`; real sub-node logic is future work
- `handle_portal_completed` writes `KillYourself<Cell>` for the portal entity
- `SurvivalPermanent` variant: like Survival but turret never self-destructs (no timer expiry)
- Portal behavior in `cells/definition/data.rs` — `CellBehavior::Portal { .. }` (no fields shown in validation match arm, grouped with Sequence as no-op validation)

---

## Confirmed Correct (as of prelude-expansion-and-import-cleanup, 2026-04-15)

- `docs/architecture/standards.md` — Prelude section fully updated: 3+ files threshold for inclusion; collision layer constants and death_pipeline types explicitly allowed; `#[cfg(test)]`-gated `test_utils` submodule allowed; 7-submodule structure documented
- `docs/architecture/testing.md` — Rule 8 updated: test infrastructure reachable via `crate::prelude` in `#[cfg(test)]` builds; direct `use crate::shared::test_utils::...` imports remain valid; code examples updated to use `DamageDealt<Cell>` / `BoltImpactCell` (replacing stale `DamageCell`)
- `docs/architecture/plugins.md` — prelude/ entry accurate: "re-exports only, no types"; cross-domain read access bullets updated: `CellHealth` → `Hp`, `DamageCell` → `DamageDealt<Cell>`, `RequestCellDestroyed`/`CellDestroyedAt` → `Destroyed<Cell>`/`Destroyed<Bolt>`/etc.
- `docs/architecture/bolt-definitions.md` — extra bolt path updated: `RequestBoltDestroyed`/`cleanup_destroyed_bolts` → `KillYourself<Bolt>` + unified death pipeline
- `docs/architecture/builders/cell.md` — `CellHealth` → `Hp` in core entity and guardian spawning steps
- `docs/architecture/content.md` — AoE effect description: `DamageCell` → `DamageDealt<Cell>`
- `docs/architecture/effects/commands.md` + `core_types.md` — `DamageCell.source_chip` → `DamageDealt.source_chip`
- `breaker-game/src/prelude/` — 8 files: mod.rs + components.rs + constants.rs + death_pipeline.rs + messages.rs + resources.rs + states.rs + test_utils.rs (last is `#[cfg(test)]` only); all pure re-export files, no type definitions
- No stale "2+ domains" or "no test_utils in prelude" or "constants stay in crate::shared" rules found anywhere in docs/architecture/

---

## Confirmed Correct (as of effect-system-refactor, 2026-04-14)

- `docs/architecture/messages.md` — removed `DamageCell`, `CellDestroyedAt`, `RequestBoltDestroyed` rows; added `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity` rows; fixed `RunLost` sender from `life_lost` to `handle_breaker_death`
- `docs/architecture/ordering.md` — removed stale `cleanup_destroyed_bolts` entry; added `DeathPipelineSystems::{ApplyDamage,DetectDeaths,HandleKill}` and `EffectV3Systems::{Tick,Conditions,Reset}` to Defined sets table; updated death bridge names (`bridge_cell_destroyed`→`on_cell_destroyed`, `bridge_bolt_death`→`on_bolt_destroyed`, added `on_wall_destroyed`/`on_breaker_destroyed`); added `.before(EffectV3Systems::Bridge)` to collision systems; added `normalize_bolt_speed_after_constraints` to bolt ordering chain; added full death pipeline chain (ApplyDamage→DetectDeaths→HandleKill) with consumer ordering; added `FixedPostUpdate` section for `process_despawn_requests`; updated prose reading section
- `docs/design/terminology/core.md` — added `Invulnerable` entry
- `docs/architecture/ordering.md` — `BoltSystems::WallCollision` entry now includes `BoltSystems::WallCollision` set assignment (was previously missing from inline chain)
- Unified death pipeline in effect: `DamageDealt<T>` → `apply_damage<T>` (ApplyDamage phase) → `detect_deaths<T>` (DetectDeaths phase) → `handle_kill<T>` (HandleKill phase) → `DespawnEntity` → `process_despawn_requests` (FixedPostUpdate)
- `handle_cell_hit`, `cleanup_cell`, `cleanup_destroyed_bolts` systems DELETED; `CellHealth` component DELETED
- `DamageCell`, `CellDestroyedAt`, `RequestBoltDestroyed`, `RequestCellDestroyed` messages DELETED
- `Invulnerable` marker component in `shared/death_pipeline/invulnerable.rs`; auto-managed by `Locked` hooks via `sync_lock_invulnerable`
- All waves A-G of plan `wiggly-swinging-pascal.md` marked DONE

---

## Standing Structural Facts

- `CleanupOnNodeExit` and `CleanupOnRunEnd` DO NOT EXIST in `breaker-game/src/`. Use `CleanupOnExit<NodeState>` and `CleanupOnExit<RunState>` from `rantzsoft_stateflow`.
- `ShieldActive` component NO LONGER EXISTS. Shield is now a timed floor wall (`ShieldWall` + `ShieldWallTimer`).
- `dispatch_breaker_effects` SUPERSEDED by `spawn_or_reuse_breaker` builder path.
- `dispatch_wall_effects` DELETED. Effect dispatch is inline in Wall builder `spawn()`.
- `SpawnAdditionalBolt` REMOVED from bolt/messages.rs — effects spawn directly via `&mut World`.
- `EffectSystems::Recalculate` REMOVED. `EffectV3Systems` has 4 variants: `Bridge`, `Tick`, `Conditions`, `Reset`.
- `DamageCell`, `CellDestroyedAt`, `RequestCellDestroyed`, `RequestBoltDestroyed` messages DELETED — replaced by `DamageDealt<T>`, `Destroyed<T>`, `KillYourself<T>`, `DespawnEntity`.
- `handle_cell_hit`, `cleanup_cell`, `cleanup_destroyed_bolts` systems DELETED — unified death pipeline handles all cell/bolt death.
- `CellHealth` component DELETED — cells use `Hp` from `shared/death_pipeline/hp.rs`.
- `RunLost` sender is `handle_breaker_death` in `RunPlugin::HandleKill`, NOT `effect/effects/life_lost`.
- All 6 `Effective*` components removed — consumers call `Active*.multiplier()` / `.total()` directly.
- `BoltSystems::InitParams` and `BoltSystems::PrepareVelocity` DO NOT EXIST.
- `BoltRadius` is a type alias for `BaseRadius` from `shared/size.rs`.
- `BoltSpeedInRange` renamed to `BoltSpeedAccurate` in invariants. InvariantKind total: 22.
- MutationKind total: 17 variants. First variant is `SetDashState`.
- `chips/components.rs` is intentionally a stub (doc comment only).
- `docs/architecture/rendering/` files are ALL forward-looking Phase 5 design docs. `rantzsoft_vfx` crate does NOT YET EXIST.
- `Salvo` implements `GameEntity` and has `apply_damage<Salvo>`, `detect_deaths<Salvo>`, `handle_kill<Salvo>` registered — but no current production sender of `DamageDealt<Salvo>` exists. Salvo-collision despawns use `commands.try_despawn()` directly.
- `CellBehavior` enum has 10 variants: `Regen`, `Guarded`, `Volatile`, `Sequence`, `Armored`, `Phantom`, `Magnetic`, `Survival`, `SurvivalPermanent`, `Portal`.
- Deferred Wave 8 doc drift (do NOT flag until after Wave 8 merges): `docs/architecture/plugins.md` domain layout table still shows `screen/`, `ui/`, `run/`, `wall/`.

For older session history, see [known-state-history.md](known-state-history.md).
