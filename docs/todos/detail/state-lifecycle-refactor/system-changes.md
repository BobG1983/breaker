# System Changes: Merges, Splits, Rewrites

Systems that need more than a gate swap or file move — they need logic changes, splitting, merging, or complete rewrites.

---

## Splits

### `select_highlights` → split into chip_select and run_end

**Current:** `run/systems/select_highlights/` — runs during chip select, picks which highlights to show on the run-end screen.

**Problem:** Conflates two concerns — mid-run highlight selection and end-of-run highlight presentation.

**Split:**
1. `state/run/chip_select/systems/select_highlights.rs` — runs during `ChipSelectState`, selects highlights from the current node's data and appends to `RunStats.highlights`
2. `state/run/run_end/systems/select_final_highlights.rs` — runs on `OnEnter(RunEndState::Active)`, selects the top N highlights for display using the diversity algorithm

### `generate_node_sequence` → keep as-is (full upfront generation)

**Current:** `run/systems/generate_node_sequence/` — generates the ENTIRE node sequence at run start. `advance_node` increments `node_index` into this pre-generated array.

**Research finding:** The current design is clean — deterministic from seed, full sequence validated at startup, three consumers read `NodeSequence.assignments[node_index]` (init_node_timer, spawn_cells_from_layout, handle_node_cleared). Refactoring to per-node generation would add complexity (mutable RNG across transitions, no upfront validation) for no clear benefit.

**Decision:** Keep as-is. System moves to `state/run/loading/systems/generate_node_sequence/`. May revisit to per-tier batching (4 nodes + boss) if run structure becomes more dynamic later, but not in this refactor.

---

## Rewrites (logic changes, not just gate swaps)

### `handle_node_cleared` — REWRITE

**Current:** Sets `NextState<GameState>(TransitionOut)`.
**After:** Sets `NextState<NodeState>(AnimateOut)`. The NodeState::Teardown routing system then determines the next RunState (ChipSelect or RunEnd) based on NodeResult / node index.

**Location:** `state/run/node/lifecycle/handle_node_cleared.rs`

### `handle_timer_expired` — REWRITE

**Current:** Sets `NextState<GameState>(RunEnd)`.
**After:** Sets `NextState<NodeState>(AnimateOut)`. A flag or resource (e.g., `RunOutcome::TimerExpired`) tells the teardown routing to go to RunEnd instead of ChipSelect.

**Location:** `state/run/node/lifecycle/handle_timer_expired.rs`

### `handle_run_lost` — REWRITE

**Current:** Sets `NextState<GameState>(RunEnd)`.
**After:** Sets `NextState<NodeState>(AnimateOut)`. Sets `RunOutcome::LivesDepleted`. Teardown routing reads RunOutcome to decide RunEnd.

**Location:** `state/run/node/lifecycle/handle_run_lost.rs`

### `handle_main_menu_input` — REWRITE

**Current:** Sets `NextState<GameState>(RunSetup)` on Play, `AppExit` on Quit.
**After:** Sets `NextState<MenuState>(StartGame)` on Play, `AppExit` on Quit.

**Location:** `state/menu/main/systems/handle_main_menu_input.rs`

### `handle_run_setup_input` — REWRITE

**Current:** Sets `NextState<GameState>(Playing)` on confirm.
**After:** Sets `NextState<MenuState>(Teardown)` on confirm. MenuState::Teardown triggers parent GameState::Menu → GameState::Run transition.

**Location:** `state/menu/start_game/systems/handle_run_setup_input.rs`

### `handle_chip_input` — REWRITE

**Current:** Sets `NextState<GameState>(TransitionIn)` on selection.
**After:** Sets `NextState<ChipSelectState>(AnimateOut)` on selection. ChipSelectState teardown triggers RunState::ChipSelect → RunState::Node.

**Location:** `state/run/chip_select/systems/handle_chip_input.rs`

### `tick_chip_timer` — REWRITE

**Current:** Sets `NextState<GameState>(TransitionIn)` on expiry.
**After:** Sets `NextState<ChipSelectState>(AnimateOut)` on expiry.

**Location:** `state/run/chip_select/systems/tick_chip_timer.rs`

### `handle_run_end_input` — REWRITE

**Current:** Sets `NextState<GameState>(MainMenu)`.
**After:** Sets `NextState<RunEndState>(AnimateOut)`. RunEndState teardown triggers RunState::RunEnd → RunState::Teardown → GameState::Run → GameState::Menu.

**Location:** `state/run/run_end/systems/handle_run_end_input.rs`

### `check_spawn_complete` — REWRITE

**Current:** Waits for BoltSpawned + BreakerSpawned, sends SpawnNodeComplete message.
**After:** Waits for BoltSpawned + BreakerSpawned + CellsSpawned + WallsSpawned (if applicable), sets `NextState<NodeState>(AnimateIn)`.

**Location:** `state/run/node/systems/check_spawn_complete.rs`

### `toggle_pause` — REWRITE

**Current:** Reads InputActions for TogglePause, sets `NextState<PlayingState>` to toggle Active↔Paused.
**After:** Reads InputActions for TogglePause, calls `time.pause()` or `time.unpause()` on `Time<Virtual>`.

**Location:** `state/pause/systems/toggle_pause.rs`

### `handle_pause_input` — REWRITE

**Current:** Reads keyboard input, navigates menu. Resume sets `NextState<PlayingState>(Active)`. Quit sets `NextState<GameState>(MainMenu)`.
**After:** Resume calls `time.unpause()`. Quit calls `time.unpause()` + sets `NextState<RunState>(Teardown)`.

**Location:** `state/pause/systems/handle_pause_input.rs`

### `spawn_pause_menu` / `cleanup<PauseMenuScreen>` — REWRITE

**Current:** OnEnter(PlayingState::Paused) / OnExit(PlayingState::Paused).
**After:** Spawn/cleanup triggered by run conditions: `is_paused() AND NOT transitioning` / `NOT is_paused()`. May need edge-detection (`condition_changed_to`) to avoid spawning every frame.

**Location:** `state/pause/systems/spawn_pause_menu.rs`

### `animate_transition` — PARKED (disabled during migration)

**Current:** Runs during GameState::TransitionOut/TransitionIn, animates overlay, sets NextState on completion.
**After:** Disabled initially. AnimateIn/AnimateOut states are instant pass-throughs. Re-enabled when lifecycle crate provides proper overlay support.

**Location:** `state/transition/system.rs` (parked)

---

## Merges

None identified. Most systems are already appropriately scoped.

---

## New Systems

### `setup_run`

**Purpose:** Spawn primary breaker + primary bolt on run start (spec @lines 441-447).
**Schedule:** OnExit(RunState::Setup)
**Logic:** Uses Breaker::builder() and Bolt::builder() to spawn entities with `CleanupOnExit<RunState>`. Reads SelectedBreaker and BoltRegistry.
**Location:** `state/run/systems/setup_run.rs`

### Routing pass-throughs

**Purpose:** Auto-advance states that don't have real content yet.
**Schedule:** OnEnter for each pass-through state
**Logic:** Each system just sets `NextState<S>` to the next variant.
**Location:** `state/routing.rs`

Pass-through states:
- `NodeState::AnimateIn` → `NodeState::Playing`
- `NodeState::AnimateOut` → `NodeState::Teardown`
- `ChipSelectState::Loading` → `ChipSelectState::AnimateIn`
- `ChipSelectState::AnimateIn` → `ChipSelectState::Selecting`
- `ChipSelectState::AnimateOut` → `ChipSelectState::Teardown`
- `RunEndState::Loading` → `RunEndState::AnimateIn`
- `RunEndState::AnimateIn` → `RunEndState::Active`
- `RunEndState::AnimateOut` → `RunEndState::Teardown`
- `MenuState::Loading` → `MenuState::Main`
- `RunState::Loading` → `RunState::Setup` (after run init systems run)

### Teardown routing

**Purpose:** On entering a Teardown state, run cleanup and determine next parent state.
**Schedule:** OnEnter(*.Teardown)
**Logic:** Each teardown system runs `cleanup_entities::<CleanupOnExit<S>>` then decides next parent state.
**Location:** `state/routing.rs` or per-state routing modules

Teardown decisions:
- `NodeState::Teardown` → reads `RunOutcome` to decide `RunState::ChipSelect` vs `RunState::RunEnd`
- `ChipSelectState::Teardown` → sets `RunState::Node`
- `RunEndState::Teardown` → sets `RunState::Teardown`
- `RunState::Teardown` → sets `GameState::Menu`
- `MenuState::Teardown` → sets `GameState::Run`

---

## Domain Cleanup After Moves

### bolt/
- Remove `spawn_bolt` system + entire `systems/spawn_bolt/` directory
- Remove `apply_node_scale_to_bolt` + `reset_bolt` from bolt plugin registration (systems move to state/)
- Drop `.after(spawn_bolt)` ordering constraints
- Keep all FixedUpdate/Update runtime systems

### breaker/
- Remove `spawn_or_reuse_breaker` from registration
- Remove `apply_node_scale_to_breaker` + `reset_breaker` from registration
- Drop `.after(spawn_or_reuse_breaker)` ordering constraints
- Keep all FixedUpdate/Update runtime systems

### cells/
- Remove `dispatch_cell_effects` from registration (moves to state/)
- Keep all FixedUpdate runtime systems

### wall/ (rename to walls/)
- Remove `spawn_walls` + `dispatch_wall_effects` from registration (move to state/)
- Wall domain becomes very thin — may just be component definitions and messages
- Consider absorbing entirely into state/run/node/ if nothing is left

### fx/
- Remove `fx/transition/` directory (moved to state/transition/)
- Remove transition system registration from fx/plugin.rs
- Keep `animate_fade_out`, `animate_punch_scale`
