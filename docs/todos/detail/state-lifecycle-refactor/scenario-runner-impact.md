# Scenario Runner Impact

Every change needed in `breaker-scenario-runner/` for the state lifecycle refactor. The scenario runner is deeply coupled to game state types, screen types, and run domain types — all of which move or change.

---

## Import Path Changes (mechanical)

Every `use breaker::` path that references moved types must be updated. These are pure path renames — no logic changes.

### GameState / PlayingState

| File | Current Import | New Import |
|------|---------------|------------|
| `invariants/types.rs:2` | `use breaker::shared::GameState` | `use breaker::state::types::GameState` (or re-export) |
| `lifecycle/systems/plugin.rs:8` | `use breaker::shared::GameState` | same |
| `lifecycle/systems/frame_control.rs:4` | `use breaker::shared::GameState` | same |
| `lifecycle/systems/menu_bypass.rs:12` | `use breaker::shared::{GameState, RunSeed}` | `use breaker::state::types::GameState` + `use breaker::state::run::resources::RunSeed` |
| `invariants/checkers/valid_state_transitions.rs:2` | `use breaker::shared::GameState` | same |
| `invariants/checkers/valid_breaker_state/checker.rs:2` | `use breaker::shared::GameState` | same |
| `invariants/checkers/physics_frozen_during_pause.rs:2` | `use breaker::shared::PlayingState` | DELETED — see Pause Rewrite below |
| `lifecycle/systems/frame_mutations/mutations.rs:18` | `use breaker::shared::{CleanupOnNodeExit, PlayingState}` | `CleanupOnNodeExit` → `CleanupOnExit<NodeState>` from state; `PlayingState` → DELETED |

### Screen / UI types

| File | Current Import | New Import |
|------|---------------|------------|
| `lifecycle/systems/plugin.rs:81` | `breaker::screen::chip_select::ChipOffers` | `breaker::state::run::chip_select::ChipOffers` |
| `lifecycle/systems/menu_bypass.rs:13` | `use breaker::ui::messages::ChipSelected` | `use breaker::state::run::chip_select::messages::ChipSelected` |
| `invariants/checkers/check_chip_offer_expected.rs:2` | `use breaker::screen::chip_select::{ChipOffering, ChipOffers}` | `use breaker::state::run::chip_select::{ChipOffering, ChipOffers}` |
| `invariants/checkers/check_offering_no_duplicates.rs:4` | `use breaker::screen::chip_select::ChipOffers` | `use breaker::state::run::chip_select::ChipOffers` |
| `invariants/checkers/check_maxed_chip_never_offered.rs:2` | `use breaker::screen::chip_select::ChipOffers` | `use breaker::state::run::chip_select::ChipOffers` |
| `lifecycle/systems/frame_mutations/mutations.rs:17` | `use breaker::screen::chip_select::{ChipOffering, ChipOffers}` | `use breaker::state::run::chip_select::{ChipOffering, ChipOffers}` |

### Run domain types

| File | Current Import | New Import |
|------|---------------|------------|
| `lifecycle/systems/frame_mutations/mutations.rs:16` | `use breaker::run::{RunStats, node::resources::NodeTimer}` | `use breaker::state::run::{RunStats, node::resources::NodeTimer}` |
| `invariants/checkers/check_run_stats_monotonic/checker.rs:2` | `use breaker::run::RunStats` | `use breaker::state::run::RunStats` |
| `lifecycle/systems/debug_setup.rs:4` | `use breaker::run::node::resources::NodeTimer` | `use breaker::state::run::node::resources::NodeTimer` |
| `lifecycle/systems/types.rs:7` | `use breaker::run::{NodeLayout, node::definition::NodePool}` | `use breaker::state::run::{NodeLayout, node::definition::NodePool}` |
| `lifecycle/systems/menu_bypass.rs:9` | `use breaker::run::{NodeLayoutRegistry, ...}` | `use breaker::state::run::{NodeLayoutRegistry, ...}` |
| `lifecycle/systems/plugin.rs:7` | `use breaker::run::node::{messages::SpawnNodeComplete, sets::NodeSystems}` | `use breaker::state::run::node::{messages::SpawnNodeComplete, sets::NodeSystems}` |
| `lifecycle/systems/plugin.rs:75` | `breaker::run::node::messages::ReverseTimePenalty` | `breaker::state::run::node::messages::ReverseTimePenalty` |
| `lifecycle/systems/frame_control.rs:4` | `use breaker::run::node::messages::SpawnNodeComplete` | `use breaker::state::run::node::messages::SpawnNodeComplete` |

### Shared types that move

| File | Current Import | New Import |
|------|---------------|------------|
| `lifecycle/systems/frame_mutations/mutations.rs:18` | `breaker::shared::CleanupOnNodeExit` | `breaker::state::cleanup::CleanupOnExit<NodeState>` |
| `invariants/checkers/check_chain_arc_count_reasonable.rs:40` | `breaker::shared::CleanupOnNodeExit` | `breaker::state::cleanup::CleanupOnExit<NodeState>` |
| `lifecycle/systems/menu_bypass.rs:12` | `breaker::shared::RunSeed` | `breaker::state::run::resources::RunSeed` |

### Shared types that stay (no change)

| Type | Import Path | Notes |
|------|-------------|-------|
| `PlayfieldConfig` | `breaker::shared::PlayfieldConfig` | Stays in shared/ — no change |
| `GameRng` | `breaker::shared::GameRng` | Stays in shared/ — no change |
| `BaseWidth` | `breaker::shared::BaseWidth` (via components) | Stays in shared/ — no change |

### SystemSet ordering (path changes only)

| File | Current Import | New Import |
|------|---------------|------------|
| `lifecycle/systems/plugin.rs:5` | `use breaker::bolt::BoltSystems` | No change (bolt/ stays) |
| `lifecycle/systems/plugin.rs:5` | `use breaker::breaker::BreakerSystems` | No change (breaker/ stays) |
| `lifecycle/systems/plugin.rs:7` | `use breaker::run::node::sets::NodeSystems` | `use breaker::state::run::node::sets::NodeSystems` |

---

## Logic Changes (rewrites needed)

### State gate migrations

All `in_state(GameState::*)` and `OnEnter(GameState::*)` references must change to the new state hierarchy.

| File:Line | Current | New |
|-----------|---------|-----|
| `plugin.rs:56` | `in_state(GameState::RunEnd)` | `in_state(RunEndState::Active)` |
| `plugin.rs:58` | `OnEnter(GameState::RunEnd)` | `OnEnter(RunEndState::Active)` |
| `plugin.rs:80` | `in_state(GameState::ChipSelect)` | `in_state(ChipSelectState::Selecting)` |
| `plugin.rs:116` | `OnEnter(GameState::MainMenu)` | `OnEnter(MenuState::Main)` |
| `plugin.rs:126` | `OnEnter(GameState::Playing)` | `OnEnter(NodeState::Loading)` |

### `bypass_menu_to_playing()` — REWRITE

**File:** `lifecycle/systems/menu_bypass.rs:30-134`

Currently: runs on `OnEnter(GameState::MainMenu)`, sets up breaker/layout/seed, then calls `next_state.set(GameState::Playing)`.

After: must navigate the full state hierarchy:
- Runs on `OnEnter(MenuState::Main)` (or `OnEnter(MenuState::Loading)`)
- Sets up breaker/layout/seed as before
- Must skip through MenuState::Main → MenuState::StartGame → MenuState::Teardown to trigger GameState::Menu → GameState::Run → RunState::Loading → RunState::Setup → RunState::Node → NodeState::Loading
- **Option A:** Set `NextState<MenuState>(Teardown)` directly — relies on teardown routing to cascade to GameState::Run. Simplest but requires several frames of state cascading.
- **Option B:** Set multiple `NextState` calls across levels — risky, only one pending per level per frame.
- **Recommended:** Option A + ensure teardown routing systems are wired. May need multiple `app.update()` calls in the runner to flush through the state cascade.

### `auto_skip_chip_select()` — REWRITE

**File:** `lifecycle/systems/menu_bypass.rs:140-157`

Currently: runs when `GameState::ChipSelect` + `ChipOffers` exists, writes `ChipSelected` message, sets `GameState::TransitionIn`.

After:
- Gated on `ChipSelectState::Selecting` + `resource_exists::<ChipOffers>`
- Writes `ChipSelected` message as before
- Sets `NextState<ChipSelectState>(AnimateOut)` (AnimateOut is a pass-through → Teardown → RunState::Node)

### `restart_run_on_end()` — REWRITE

**File:** `lifecycle/systems/frame_control.rs:43-49`

Currently: runs on `OnEnter(GameState::RunEnd)`, sets `GameState::MainMenu`.

After:
- Runs on `OnEnter(RunEndState::Active)`
- Sets `NextState<RunEndState>(AnimateOut)` → cascades through teardown to GameState::Menu

### `exit_on_run_end()` — REWRITE

**File:** `lifecycle/systems/frame_control.rs:37-42`

Currently: gated on `in_state(GameState::RunEnd)`, writes `AppExit`.

After: gated on `in_state(RunEndState::Active)`, writes `AppExit`. Minimal change.

### `map_forced_game_state()` — REWRITE

**File:** `lifecycle/systems/frame_control.rs` (if exists)

Any mapping of scenario `ForcedState` enum values to game states must be updated to reference the new state hierarchy.

### Pause mutation (`PauseControl`) — REWRITE

**File:** `lifecycle/systems/frame_mutations/mutations.rs:35-37, 170-176`

Currently: reads `State<PlayingState>`, writes `NextState<PlayingState>`, toggles Active↔Paused.

After: `PlayingState` is deleted. Pause uses `Time<Virtual>::pause()/unpause()`.
- Replace `State<PlayingState>` with `Res<Time<Virtual>>` to check `is_paused()`
- Replace `NextState<PlayingState>` with `ResMut<Time<Virtual>>` to call `pause()`/`unpause()`
- Toggle logic: `if time.is_paused() { time.unpause() } else { time.pause() }`

### `CleanupOnNodeExit` usage — UPDATE

**Files:** `frame_mutations/mutations.rs:192,194`, `check_chain_arc_count_reasonable.rs:40`

Replace `CleanupOnNodeExit` component with `CleanupOnExit<NodeState>`. These are places where the scenario runner spawns entities that need node-scoped cleanup.

### `PreviousGameState` + `ForcedGameState` + `valid_state_transitions` — DELETE (no replacement)

**Files to delete:**
- `invariants/types.rs:81-83` — `PreviousGameState` resource
- `types/definitions/scenario.rs:19-38` — `ForcedGameState` enum
- `types/definitions/scenario.rs:62` — `force_previous_game_state` field on `DebugSetup`
- `lifecycle/systems/frame_control.rs:56-68` — `map_forced_game_state()` function
- `invariants/checkers/valid_state_transitions.rs` — entire invariant checker
- `lifecycle/systems/debug_setup.rs:92-96` — `force_previous_game_state` application block
- `lifecycle/tests/debug_setup/state_mapping.rs` — all mapping tests

**Why no replacement:** The new hierarchical state machine makes invalid transitions structurally impossible — SubStates only exist when their parent is in the right variant, and the lifecycle crate's routing table is the single source of truth for legal transitions. A runtime invariant checking the same thing is redundant. Any corresponding self-test scenarios for this invariant are also deleted.

### `physics_frozen_during_pause` invariant — REWRITE

**File:** `invariants/checkers/physics_frozen_during_pause.rs`

Currently reads `State<PlayingState>` to check if Active vs Paused. After: check `Time<Virtual>::is_paused()` instead.

---

## State Registration in Scenario Runner

**File:** `runner/app.rs`

The scenario runner uses `Game::headless()` which adds `StatePlugin` (via `ScreenPlugin` currently). After the refactor, `StatePlugin` registers the new state hierarchy. The runner doesn't register states itself — it inherits them from the game plugin.

However, the runner DOES register additional messages that the game plugin also registers. Ensure no double-registration after the refactor:
- `plugin.rs:72-75` registers `SpawnNodeComplete`, `ChipSelected`, `ReverseTimePenalty` — verify these aren't already registered by `StatePlugin`.

---

## `entered_playing` Gate — UPDATE

**File:** `lifecycle/systems/frame_control.rs:75-92`

Currently: `mark_entered_playing_on_spawn_complete()` reads `SpawnNodeComplete` message and sets `ScenarioStats.entered_playing = true`. This gates invariant checkers so they don't fire during loading.

After: `SpawnNodeComplete` may be replaced or its semantics may change (see `system-changes.md` — `check_spawn_complete` REWRITE sets `NextState<NodeState>(AnimateIn)` instead of sending `SpawnNodeComplete`).

**Decision needed:** Keep `SpawnNodeComplete` message for the scenario runner's benefit (even if the game's `check_spawn_complete` no longer sends it), OR change the gate to check `in_state(NodeState::Playing)` instead. The latter is cleaner — the invariant gate becomes a simple state check rather than a message-based flag.

---

## Summary: Files That Need Changes

| File | Change Type | Scope |
|------|------------|-------|
| `lifecycle/systems/plugin.rs` | REWRITE | State gates, OnEnter schedules, message registration, system ordering |
| `lifecycle/systems/menu_bypass.rs` | REWRITE | State transitions, bypass flow for new hierarchy |
| `lifecycle/systems/frame_control.rs` | REWRITE | State references, restart logic, entered_playing gate |
| `lifecycle/systems/frame_mutations/mutations.rs` | REWRITE | Pause mutation (Time<Virtual>), CleanupOnNodeExit→CleanupOnExit, import paths |
| `lifecycle/systems/debug_setup.rs` | PATH CHANGE | NodeTimer import path |
| `lifecycle/systems/types.rs` | PATH CHANGE | NodeLayout import path |
| `invariants/types.rs` | DELETE field | `PreviousGameState` resource deleted (no replacement) |
| `invariants/checkers/valid_state_transitions.rs` | DELETE | Entire invariant deleted — hierarchy makes invalid transitions structural |
| `invariants/checkers/valid_breaker_state/checker.rs` | PATH CHANGE | GameState import |
| `invariants/checkers/physics_frozen_during_pause.rs` | REWRITE | Time<Virtual>::is_paused() replaces PlayingState |
| `invariants/checkers/check_chip_offer_expected.rs` | PATH CHANGE | ChipOffers import |
| `invariants/checkers/check_offering_no_duplicates.rs` | PATH CHANGE | ChipOffers import |
| `invariants/checkers/check_maxed_chip_never_offered.rs` | PATH CHANGE | ChipOffers import |
| `invariants/checkers/check_run_stats_monotonic/checker.rs` | PATH CHANGE | RunStats import |
| `invariants/checkers/check_chain_arc_count_reasonable.rs` | UPDATE | CleanupOnNodeExit → CleanupOnExit<NodeState> |
| `runner/app.rs` | CLEANUP | Remove duplicate `add_message` calls for SpawnNodeComplete, ChipSelected, ReverseTimePenalty (game plugin already registers these) |
| `types/definitions/scenario.rs` | DELETE field + enum | `ForcedGameState` enum and `force_previous_game_state` field deleted — no longer needed |
| `types/definitions/mutations.rs` | REWRITE | `MutationKind::TogglePause` switches to Time<Virtual>. `InjectDuplicateOffers`/`InjectMaxedChipOffer` need ChipOffers import path update. |
| `lifecycle/tests/debug_setup/state_mapping.rs` | DELETE | `map_forced_game_state` tests deleted with the function |
| `lifecycle/systems/frame_control.rs:56-68` | DELETE fn | `map_forced_game_state()` deleted |
| `lifecycle/systems/debug_setup.rs:92-96` | DELETE block | `force_previous_game_state` application deleted |

**Total: 18 files need changes.** ~7 path-only, ~9 rewrites, ~2 cleanup.

---

## Dead Messages

### `SpawnNodeComplete` — DELETE

- **Produced by:** `check_spawn_complete` in game crate
- **Consumed by:** NOBODY in game crate. Only the scenario runner reads it (for `entered_playing` gate).
- **After refactor:** `check_spawn_complete` sets `NextState<NodeState>(AnimateIn)` directly. The scenario runner should switch its `entered_playing` gate to `in_state(NodeState::Playing)` — simpler and doesn't depend on a game-side message that serves no game purpose.
- **Action:** Delete `SpawnNodeComplete` message, remove `add_message` registration, update scenario runner to use state check.

All other messages remain needed — they carry gameplay-significant data consumed by game systems.

---

## Routing Table Hack for Fast-Path

Instead of complex `bypass_menu_to_playing` rewrite, the scenario runner overrides routes in the `RoutingTable` to skip states that wait for user input:

- Override `Route::from(MenuState::Main).to(MenuState::Teardown)` — skip menu interaction
- Override `Route::from(ChipSelectState::Selecting).to(ChipSelectState::Teardown)` — skip chip selection (after writing `ChipSelected` message if configured)
- Override `Route::from(RunEndState::Active).to(RunEndState::Teardown)` — skip run end acknowledgement (for restart loops)

The normal pass-through routing (AnimateIn→Playing, AnimateOut→Teardown, Loading→next) handles all intermediate states. The runner just skips states that wait for user input. Each pass-through costs one frame — fine for headless simulation.

`bypass_menu_to_playing` simplifies to: set up breaker/layout/seed resources, override routes, send `ChangeState<MenuState>` to kick off the cascade.

`auto_skip_chip_select` simplifies to: the overridden route already skips Selecting. The runner just needs to write the `ChipSelected` message on `OnEnter(ChipSelectState::Selecting)` if a chip is configured.

`restart_run_on_end` simplifies to: the overridden route already skips RunEndState::Active. The cascade naturally flows back to MenuState::Main (which the runner also skips).

**Test coverage:** Unit tests verify the runner's route overrides produce the expected state cascade. If the state hierarchy changes, these tests fail — catching the breakage.

### `entered_playing` gate — use `OnEnter(NodeState::Playing)`

Replace `mark_entered_playing_on_spawn_complete()` (reads `SpawnNodeComplete` message) with a simple `in_state(NodeState::Playing)` run condition. By the time `NodeState::Playing` is entered, all spawning and setup is guaranteed complete — the Loading→AnimateIn→Playing sequence ensures this structurally.

---

## Deleted Invariants and Scenarios

### `ValidStateTransitions` invariant — DELETE

The invariant, its `InvariantKind` variant, and the `PreviousGameState` resource are all deleted. Invalid transitions are structurally prevented by the SubStates hierarchy.

### `invalid_state_transition` scenario — DELETE

`scenarios/self_tests/invalid_state_transition.scenario.ron` exists solely to self-test `ValidStateTransitions` using `force_previous_game_state: Loading`. Both the invariant and the forced state mechanism are deleted.

### ~30 scenarios — remove `ValidStateTransitions` from `disallowed_failures`

Every scenario that lists `ValidStateTransitions` in `disallowed_failures` needs that entry removed. This is mechanical — delete the variant from each RON file's list. The scenarios themselves remain valid (they test other invariants like BoltInBounds, NoNaN, etc.).

### `PhysicsFrozenDuringPause` invariant — DELETE (separate todo)

Unit tests can verify `Time<Virtual>::pause()` freezes FixedUpdate — a scenario invariant is overkill for this. Tracked as a separate todo item. During this refactor: delete the invariant and its `InvariantKind` variant. `aegis_pause_stress.scenario.ron` removes this from its `disallowed_failures` list (or is deleted if that was its only purpose).

---

## Duplicate Message Registration — CLEANUP

`plugin.rs:72-75` registers `SpawnNodeComplete` (being deleted), `ChipSelected`, and `ReverseTimePenalty`. These are already registered by the game plugin via `Game::headless()`. After the refactor:
- Remove `SpawnNodeComplete` registration (message deleted)
- Remove `ChipSelected` and `ReverseTimePenalty` registrations (game plugin handles these)

