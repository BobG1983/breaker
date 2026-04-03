# Implementation Waves

Ordered implementation plan for the state lifecycle refactor. Each wave is a branch. Sub-waves within a wave can sometimes run in parallel (see Parallelism section at the end).

**Reference files:**
- `system-moves.md` — where each system moves
- `system-changes.md` — merges, splits, rewrites
- `post-restructure-tree.md` — expected folder tree after Wave 2
- `state-assignments.md` — current state vs target state for each system
- `crate-design.md` — rantzsoft_lifecycle crate design
- `routing-tables.md` — each state's routing implementation
- `crate-migration.md` — systems that need updating for lifecycle crate

---

## Wave 1: Folder Restructure (file moves only, no logic changes)

**Branch:** `refactor/state-folder-structure`

Move files to their new homes WITHOUT changing any code. Every `use` path, every `mod` declaration, every `pub(crate)` visibility — updated for the new locations. Zero behavior changes. The game must compile and all tests must pass after this wave.

### Wave 1a: Create `src/state/` skeleton + move screen/ and ui/

Create the `state/` module structure. Move screen sub-plugins and ui systems to their new state/ locations. Update all `mod` declarations and `use` paths.

**What moves (see `system-moves.md` "From screen/" and "From ui/" sections):**
- `screen/loading/` → `state/app/loading/`
- `screen/main_menu/` → `state/menu/main/`
- `screen/run_setup/` → `state/menu/start_game/`
- `screen/chip_select/` → `state/run/chip_select/`
- `screen/run_end/` → `state/run/run_end/`
- `screen/pause_menu/` → `state/pause/`
- `screen/systems/cleanup.rs` → `state/cleanup.rs`
- `screen/plugin.rs` → `state/plugin.rs`
- `ui/systems/*` → `state/run/node/hud/systems/`
- `ui/components.rs` → `state/run/node/hud/components.rs`
- `ui/resources.rs` → `state/run/node/hud/resources.rs`
- `ui/sets.rs` → `state/run/node/hud/sets.rs`
- `ui/messages.rs` (ChipSelected) → `state/run/chip_select/messages.rs`
- `fx/transition/` → `state/transition/`

**What's deleted after move:**
- `src/screen/` directory
- `src/ui/` directory
- `src/fx/transition/` directory

**What's updated:**
- `lib.rs` — add `pub mod state;`, remove `pub mod screen;`, `pub mod ui;`
- `game.rs` — replace `ScreenPlugin` + `UiPlugin` with `StatePlugin`
- All cross-crate imports in `breaker-scenario-runner/` that reference `breaker::screen::*` or `breaker::ui::*`
- All domain plugin imports that reference `crate::shared::GameState`, `crate::shared::PlayingState`, `crate::ui::*`

### Wave 1b: Move run/ domain into state/run/

Move the entire `run/` domain into `state/run/`. This is a large move — 90+ files including tests.

**What moves (see `system-moves.md` "From run/" section):**
- `run/plugin.rs` → `state/run/plugin.rs`
- `run/resources/` → `state/run/resources/`
- `run/components.rs` → `state/run/components.rs`
- `run/messages.rs` → `state/run/messages.rs`
- `run/definition/` → `state/run/definition/`
- `run/systems/*` → distributed across `state/run/loading/`, `state/run/systems/`, `state/run/node/lifecycle/`, `state/run/node/tracking/`, `state/run/node/highlights/`, `state/run/chip_select/`, `state/run/run_end/`
- `run/highlights/` → `state/run/node/highlights/`
- `run/node/*` → `state/run/node/`

**What's deleted:** `src/run/` directory

**What's updated:**
- `lib.rs` — remove `pub mod run;`
- `game.rs` — remove `RunPlugin` (absorbed into `StatePlugin`)
- All domain imports referencing `crate::run::*` → `crate::state::run::*`

### Wave 1c: Move setup systems from bolt/breaker/cells/wall into state/run/node/

Move OnEnter setup systems that are about state setup, not domain runtime.

**What moves (see `system-moves.md` "From bolt/→state/" etc. sections):**
- `bolt/systems/apply_node_scale_to_bolt.rs` → `state/run/node/systems/`
- `bolt/systems/reset_bolt/` → `state/run/node/systems/reset_bolt/`
- `breaker/systems/apply_node_scale_to_breaker.rs` → `state/run/node/systems/`
- `breaker/systems/spawn_breaker/` (reset_breaker fn) → `state/run/node/systems/reset_breaker.rs`
- `cells/systems/dispatch_cell_effects.rs` → `state/run/node/systems/`
- `wall/systems/spawn_walls.rs` → `state/run/node/systems/`
- `wall/systems/dispatch_wall_effects.rs` → `state/run/node/systems/`

**What's updated:**
- `bolt/plugin.rs` — remove setup system registrations, drop `.after(spawn_bolt)` ordering
- `breaker/plugin.rs` — remove setup system registrations, drop `.after(spawn_or_reuse_breaker)` ordering
- `cells/plugin.rs` — remove `dispatch_cell_effects` registration
- `wall/plugin.rs` — remove `spawn_walls`/`dispatch_wall_effects` registration

**What's deleted:**
- `bolt/systems/spawn_bolt/` — entire directory (replaced by setup_run in Wave 2)
- `breaker/systems/spawn_or_reuse_breaker` fn (keep reset_breaker which moved)
- Rename `wall/` → `walls/`

### Wave 1d: Move shared state types to state/types/

- `shared/game_state.rs` → `state/types/game_state.rs` (content unchanged for now — still old 9-variant enum)
- `shared/playing_state.rs` → `state/types/playing_state.rs` (still exists, used by old gates)
- `shared/components.rs` — remove `CleanupOnNodeExit`, `CleanupOnRunEnd` (move to `state/cleanup.rs`)
- `shared/resources.rs` (RunSeed) → `state/run/resources/`
- Update all imports throughout the codebase

**Post Wave 1 verification:** `cargo dcheck`, `cargo all-dtest`, `cargo scenario -- --all` — everything must compile and pass with zero behavior changes.

---

## Wave 2: System Merges, Splits, and New Systems

**Branch:** `refactor/system-changes`

Create new systems, split/merge existing ones. Still using OLD state types — no state migration yet.

### Wave 2a: Create `setup_run` system

New system in `state/run/systems/setup_run.rs`. See `system-changes.md` "New Systems" section.

- Runs on `OnExit(RunState::Setup)` (wired later in Wave 4)
- For now, can be tested standalone with a headless app
- Spawns primary breaker + bolt via builders with `CleanupOnExit<RunState>` (or current cleanup markers until Wave 4)
- Delete `spawn_bolt` and `spawn_or_reuse_breaker` systems (already moved/removed in Wave 1c)

### Wave 2b: Split `select_highlights`

See `system-changes.md` "Splits" section.

- Split into `state/run/chip_select/systems/select_highlights.rs` (mid-run selection)
- And `state/run/run_end/systems/select_final_highlights.rs` (end-of-run presentation)

### Wave 2c: Refactor `generate_node_sequence` (optional)

Evaluate whether to keep full-sequence-at-run-start or switch to per-node/per-tier generation. See research agent findings. If refactoring, do it now before state migration adds complexity.

### Wave 2d: Create `CleanupOnExit<S>` component

In `state/cleanup.rs`, define the generic cleanup component. For now, it coexists with old `CleanupOnNodeExit`/`CleanupOnRunEnd`.

**Post Wave 2 verification:** `cargo dcheck`, `cargo all-dtest` — new systems compile, splits work, no regressions.

---

## Wave 3: Define New State Types

**Branch:** `feature/state-types`

Define all 7 new state enums alongside the old ones.

### Wave 3a: Write state enum definitions

Create files in `state/types/` for: `AppState`, new `GameState` (4 variants), `MenuState`, `RunState`, `NodeState`, `ChipSelectState`, `RunEndState`.

See `post-restructure-tree.md` "state/types/" section for exact enum definitions.

### Wave 3b: Register new states

In `state/plugin.rs`, register all new states via `init_state::<AppState>()` + `add_sub_state` chain. Keep old state registration alive temporarily.

Registration order (parent-first): AppState → GameState → MenuState, RunState → NodeState, ChipSelectState, RunEndState.

**Post Wave 3 verification:** `cargo dcheck` — dual state registration compiles. Old tests still pass.

---

## Wave 4: Migrate to New States

**Branch:** `refactor/state-migration`

The big one. Move every system from old state gates to new state gates. Implement pass-through routing. Delete old states. See `state-assignments.md` for the complete before/after mapping.

### Wave 4a: Create routing systems

Write `state/routing.rs` with all pass-through and teardown routing systems. See `routing-tables.md` for every route.

- Pass-throughs: AnimateIn→Playing, AnimateOut→Teardown, Loading→next, etc.
- Teardown routers: read RunOutcome, determine next parent state
- Cleanup systems: `cleanup_entities::<CleanupOnExit<S>>` on Teardown entry

### Wave 4b: Migrate domain plugin gates

Mechanical replacement across all domain plugins. See `state-assignments.md` for exact mappings.

- All `PlayingState::Active` → `NodeState::Playing` (42+ systems)
- All `OnEnter(GameState::Playing)` → `OnEnter(NodeState::Loading)` (setup systems)
- All `GameState::ChipSelect` → `ChipSelectState::Selecting`
- All `GameState::RunEnd` → `RunEndState::Active`
- All `GameState::MainMenu` → `MenuState::Main`
- All `GameState::RunSetup` → `MenuState::StartGame`
- All `GameState::Loading` → `AppState::Loading`
- Effect triggers: 18 files, each 1 line change
- Effect effects: 10 files, each 1 line change

### Wave 4c: Rewrite transition-setting systems

Systems that call `next_state.set()` with old state variants get rewritten. See `system-changes.md` "Rewrites" section.

- `handle_node_cleared` — set NodeState::AnimateOut + RunOutcome::InProgress
- `handle_timer_expired` — set NodeState::AnimateOut + RunOutcome::TimerExpired
- `handle_run_lost` — set NodeState::AnimateOut + RunOutcome::LivesDepleted
- `handle_main_menu_input` — set MenuState::StartGame
- `handle_run_setup_input` — set MenuState::Teardown
- `handle_chip_input` — set ChipSelectState::AnimateOut
- `tick_chip_timer` — set ChipSelectState::AnimateOut
- `handle_run_end_input` — set RunEndState::AnimateOut
- `check_spawn_complete` — set NodeState::AnimateIn

### Wave 4d: Rewrite pause system

Replace PlayingState with Time<Virtual>::pause(). See `system-changes.md` "Rewrites" section.

- `toggle_pause` → `Time<Virtual>::pause()/unpause()`
- `spawn_pause_menu` → run condition based on `is_paused()`
- `handle_pause_input` → `time.unpause()` for resume, `NextState<RunState>(Teardown)` for quit
- `cleanup<PauseMenuScreen>` → edge-triggered on unpause

### Wave 4e: Delete old states and cleanup markers

- Delete old `GameState` (9-variant version) from `state/types/`
- Delete `PlayingState` entirely
- Delete `CleanupOnNodeExit`, `CleanupOnRunEnd` from `shared/components.rs`
- Replace all usages with `CleanupOnExit<NodeState>` / `CleanupOnExit<RunState>`
- Update all test helpers that register old states

### Wave 4f: Update test helpers

Every `plugin_builds` test and integration test that does:
```rust
.init_state::<GameState>()
.add_sub_state::<PlayingState>()
```
Changes to register the new state hierarchy. This is mechanical but touches 15+ test files.

**Post Wave 4 verification:** `cargo all-dtest`, `cargo scenario -- --all` — full gameplay working on new state hierarchy with plain Bevy routing.

---

## Wave 5: Build rantzsoft_lifecycle Crate

**Branch:** `feature/lifecycle-crate`

TDD the entire crate. See `crate-design.md` for the full specification.

### Wave 5a: Crate skeleton + messages

- Create `rantzsoft_lifecycle/` workspace member
- Cargo.toml, lib.rs, plugin.rs skeleton
- Cargo aliases: `lifecycletest`, `lifecycleclippy`, `lifecyclecheck`
- `ChangeState<S>`, `StateChanged<S>` message types
- Tests for message generic instantiation

### Wave 5b: Route builder + routing table

- `Route::from(S).to(S)` / `.to_dynamic(fn)` builder
- `RoutingTable<S>` resource
- `add_route` returning Result, duplicate detection
- Tests for builder API, duplicate detection

### Wave 5c: Message-triggered dispatch

- Exclusive dispatch system gated by `on_message::<ChangeState<S>>()`
- `Commands::run_system` for dynamic routes
- Static route resolution
- First-message-only semantics, warn on duplicates
- Tests for dispatch, dynamic routing, edge cases

### Wave 5d: `when()` polling

- `.when(fn(&World) -> bool)` on route builder
- Condition-triggered dispatch system (runs every frame, evaluates conditions)
- Mutual exclusivity with message trigger
- Tests

### Wave 5e: `CleanupOnExit<S>`

- Generic marker component
- `cleanup_on_exit::<S>` system
- `app.register_cleanup::<S>(teardown_variant)` helper
- Tests

### Wave 5f: Transition overlay system

- Marker traits: `InTransition`, `OutTransition`, `OneShotTransition`
- `TransitionType` enum
- `TransitionRegistry` (TypeId → starter closure)
- Marker resources: `StartingTransition<T>`, `RunningTransition<T>`, `EndingTransition<T>`
- Internal messages: `TransitionReady`, `TransitionRunComplete`, `TransitionOver`
- Outward messages: `TransitionStart<S>`, `TransitionEnd<S>`
- Orchestration lifecycle system
- `Time<Virtual>` pause/unpause during transitions
- Deferred `ChangeState<S>` while transition active
- `.with_transition()` / `.with_dynamic_transition()` on route builder
- `.register_custom_transition::<T>()` on plugin builder
- Startup validation: panic on unregistered transition references
- Built-in effects: FadeIn, FadeOut
- Extensive tests for full lifecycle

### Wave 5g: Plugin integration + validation

- `RantzLifecyclePlugin` builder pattern
- Per-state-type generic registration
- Startup route validation
- Integration tests

**Post Wave 5 verification:** `cargo lifecycletest` — all crate tests pass. `cargo lifecycleclippy` — clean. `cargo all-dtest` — no regressions (game doesn't use crate yet).

---

## Wave 6: Wire Routing Tables to Crate

**Branch:** `feature/wire-lifecycle-routes`

Each state writes its routing table using `app.add_route()`. See `routing-tables.md` for exact routes.

### Wave 6a: Add crate dependency + register plugin

- `breaker-game/Cargo.toml` — add `rantzsoft_lifecycle` dependency
- `game.rs` — add `RantzLifecyclePlugin` to plugin group (before `StatePlugin`)
- Register all message types: `ChangeState<NodeState>`, `ChangeState<RunState>`, etc.

### Wave 6b: Write route definitions

In `state/plugin.rs` (or per-state plugin files), add all `app.add_route()` calls from `routing-tables.md`. Keep plain Bevy routing systems alive for now — routes are registered but not yet the sole routing mechanism.

### Wave 6c: Test route registration

Verify all routes are correctly registered and startup validation passes.

**Post Wave 6 verification:** `cargo dtest` — game compiles with routes registered alongside old routing.

---

## Wave 7: Migrate to Crate Routing

**Branch:** `refactor/crate-routing-migration`

Switch from plain Bevy NextState::set to ChangeState messages and crate routing. See `crate-migration.md`.

### Wave 7a: Switch game systems to ChangeState messages

Update every system listed in `crate-migration.md` "Systems that call NextState::set" to send `ChangeState<S>` messages instead.

### Wave 7b: Remove plain routing systems

Delete `state/routing.rs` — all pass-throughs and teardown routers replaced by routing table.

### Wave 7c: Wire transitions to routes

Add `.with_transition()` calls to routes that should have visual transitions. Delete `state/transition/` (parked systems replaced by crate overlay).

### Wave 7d: Cleanup

- Remove old `CleanupOnExit<S>` from `state/cleanup.rs` — re-export from crate
- Remove any remaining vestiges of plain Bevy routing

**Post Wave 7 verification:** Full verification tier — `cargo all-dtest`, `cargo scenario -- --all`, linting, all reviewers.

---

## Wave 8: Architecture Docs

**Branch:** `docs/state-architecture`

See `state-lifecycle-refactor.md` "Architecture Docs" section for the full list of new and updated docs.

---

## Parallelism

| Waves | Can parallelize? | Notes |
|-------|-----------------|-------|
| 1a, 1b | Yes | Independent file moves (different directories) |
| 1c, 1d | Yes | Independent from each other, but depend on 1a/1b |
| 2a, 2b, 2c, 2d | Yes | Independent new systems/changes |
| 3a, 3b | Sequential | 3b depends on 3a |
| 4a through 4f | Mostly sequential | 4a first, then 4b-4c can parallel, 4d independent, 4e-4f after 4b-4d |
| 5a through 5g | Mostly sequential | 5a first, then 5b-5e can somewhat parallel, 5f depends on 5b-5c, 5g last |
| 6a, 6b, 6c | Sequential | Each depends on previous |
| 7a through 7d | Mostly sequential | 7a first, then 7b-7c can parallel, 7d last |
| Wave 5 vs Wave 4 | **Wave 5 can start during Wave 4** | Crate is game-agnostic, can be built while game migration is in progress |

**Recommended parallel execution:**
```
Wave 1a + 1b (parallel)
  → Wave 1c + 1d (parallel)
    → Wave 2a + 2b + 2c + 2d (parallel)
      → Wave 3
        → Wave 4a
          → Wave 4b + 4c + 4d (parallel)  ←→  Wave 5a-5g (lifecycle crate, parallel track)
            → Wave 4e + 4f
              → Wave 6
                → Wave 7
                  → Wave 8
```
