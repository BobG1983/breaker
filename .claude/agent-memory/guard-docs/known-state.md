---
name: known-state
description: Confirmed doc/code alignment state for current and recent sessions; older session history in known-state-history.md
type: project
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
- `tiling.rs` — pure grid math for parallel visual-mode window placement; `TilePosition`, env vars `SCENARIO_WINDOW_X/Y/W/H`
- `coverage.rs` — `CoverageReport`, `check_coverage()`, `print_coverage_report()`; prints gaps only
- `discovery.rs` — RON parsed with `ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME)` (the "RON parse dedup" is really IMPLICIT_SOME extension)

## Intentionally forward-looking / do NOT flag

- `cargo.md` scenario runner options table — does not list `--fail-fast`, `--no-fail-fast`, `--clean`, `--coverage` flags. These were added on feature/scenario-runner-wiring but `.claude/rules/cargo.md` is not in guard-docs' edit scope. Needs human update.

---

## Confirmed Correct (as of prelude refactor, 2026-04-06)

- `docs/architecture/standards.md` — Prelude section: submodule/glob threshold distinction (2+ for submodules, 3+ for curated glob) is now documented correctly
- `docs/architecture/plugins.md` — prelude/ entry in Domain Layout matches actual prelude structure (re-exports only, no types)
- `breaker-game/src/prelude/` — 5 files: mod.rs + components.rs + messages.rs + resources.rs + states.rs; all pure re-export files, no type definitions

---

## Standing Structural Facts

- `CleanupOnNodeExit` and `CleanupOnRunEnd` DO NOT EXIST in `breaker-game/src/`. Use `CleanupOnExit<NodeState>` and `CleanupOnExit<RunState>` from `rantzsoft_stateflow`.
- `ShieldActive` component NO LONGER EXISTS. Shield is now a timed floor wall (`ShieldWall` + `ShieldWallTimer`).
- `dispatch_breaker_effects` SUPERSEDED by `spawn_or_reuse_breaker` builder path.
- `dispatch_wall_effects` DELETED. Effect dispatch is inline in Wall builder `spawn()`.
- `SpawnAdditionalBolt` REMOVED from bolt/messages.rs — effects spawn directly via `&mut World`.
- `EffectSystems::Recalculate` REMOVED. `EffectSystems` has only `Bridge`.
- All 6 `Effective*` components removed — consumers call `Active*.multiplier()` / `.total()` directly.
- `BoltSystems::InitParams` and `BoltSystems::PrepareVelocity` DO NOT EXIST.
- `BoltRadius` is a type alias for `BaseRadius` from `shared/size.rs`.
- `BoltSpeedInRange` renamed to `BoltSpeedAccurate` in invariants. InvariantKind total: 22.
- MutationKind total: 17 variants. First variant is `SetDashState`.
- `chips/components.rs` is intentionally a stub (doc comment only).
- `docs/architecture/rendering/` files are ALL forward-looking Phase 5 design docs. `rantzsoft_vfx` crate does NOT YET EXIST.
- Deferred Wave 8 doc drift (do NOT flag until after Wave 8 merges): `docs/architecture/plugins.md` domain layout table still shows `screen/`, `ui/`, `run/`, `wall/`.

For older session history, see [known-state-history.md](known-state-history.md).
