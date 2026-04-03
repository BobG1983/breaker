# Routing Tables

Each state's routing implementation. Initially plain Bevy (`NextState::set`), later migrated to lifecycle crate (`ChangeState<S>` messages + `when()` conditions).

For the initial migration (Wave 4), these are implemented as OnEnter systems in `state/routing.rs`. For the lifecycle crate migration (Wave 7), they become `app.add_route()` calls.

---

## AppState

| From | To | Trigger | Transition | Notes |
|------|----|---------|------------|-------|
| Loading | Game | ProgressPlugin completes | FadeIn | Disk assets loaded |
| Game | *(sub-states take over)* | — | — | GameState::Loading is default |
| Teardown | *(app shutdown)* | — | — | Not used in normal flow |

## GameState (sub-state of AppState::Game)

| From | To | Trigger | Transition | Notes |
|------|----|---------|------------|-------|
| Loading | Menu | Registry stuffing complete | FadeIn | Second loading phase |
| Menu | Run | `when()`: MenuState reached Teardown | OutIn(FadeOut, FadeIn) | Parent watches child teardown |
| Run | Menu | `when()`: RunState reached Teardown | OutIn(FadeOut, FadeIn) | Run ended, back to menu |
| Teardown | *(not used)* | — | — | |

**Signals used:**
- Menu→Run: `MenuState::Teardown` entered (detected via `condition_changed_to(false, state_exists::<MenuState>())` or resource flag)
- Run→Menu: `RunState::Teardown` entered (same pattern)

## MenuState (sub-state of GameState::Menu)

| From | To | Trigger | Transition | Notes |
|------|----|---------|------------|-------|
| Loading | Main | Immediate pass-through | none | |
| Main | StartGame | `handle_main_menu_input` confirms Play | SlideLeft (OneShot) | |
| Main | Options | `handle_main_menu_input` confirms Options | SlideLeft (OneShot) | |
| Main | Meta | `handle_main_menu_input` confirms Meta | SlideLeft (OneShot) | |
| StartGame | Teardown | `handle_run_setup_input` confirms breaker | none | Parent GameState watches |
| Options | Main | Back button | SlideRight (OneShot) | |
| Meta | Main | Back button | SlideRight (OneShot) | |
| Teardown | *(no route)* | — | — | Parent GameState::Menu when() fires |

**Signals used:**
- All transitions triggered by user input → systems call `NextState<MenuState>::set()` (later `ChangeState<MenuState>`)

## RunState (sub-state of GameState::Run)

| From | To | Trigger | Transition | Notes |
|------|----|---------|------------|-------|
| Loading | Setup | Run init complete (reset_run_state, generate_sequence, capture_seed) | none | |
| Setup | Node | `setup_run` on OnExit spawns breaker+bolt | FadeIn | First node begins |
| Node | ChipSelect | `when()`: NodeState reached Teardown AND RunOutcome == InProgress AND NOT final node | OutIn(FadeOut, FadeIn) | |
| Node | RunEnd | `when()`: NodeState reached Teardown AND (RunOutcome != InProgress OR final node win) | OutIn(FadeOut, FadeIn) | |
| ChipSelect | Node | `when()`: ChipSelectState reached Teardown | OutIn(FadeOut, FadeIn) | advance_node on OnEnter(Node) |
| RunEnd | Teardown | `when()`: RunEndState reached Teardown | none | |
| Teardown | *(no route)* | — | — | Parent GameState::Run when() fires |

**Key routing decision — Node teardown:**
The `node_teardown_router` system reads `RunOutcome` resource to decide:
- `RunOutcome::InProgress` + not final node → `RunState::ChipSelect`
- `RunOutcome::Won` (final node cleared) → `RunState::RunEnd`
- `RunOutcome::TimerExpired` → `RunState::RunEnd`
- `RunOutcome::LivesDepleted` → `RunState::RunEnd`

**Signals used:**
- Node→ChipSelect/RunEnd: `RunOutcome` resource (set by `handle_node_cleared`, `handle_timer_expired`, `handle_run_lost`)
- ChipSelect→Node: ChipSelectState teardown (player selected or timer expired)
- RunEnd→Teardown: RunEndState teardown (player acknowledged)

## NodeState (sub-state of RunState::Node)

| From | To | Trigger | Transition | Notes |
|------|----|---------|------------|-------|
| Loading | AnimateIn | `check_spawn_complete` (all spawn signals received) | none | |
| AnimateIn | Playing | Pass-through (instant, real animation later) | none | |
| Playing | AnimateOut | `handle_node_cleared` OR `handle_timer_expired` OR `handle_run_lost` | none | These systems set RunOutcome first |
| AnimateOut | Teardown | Pass-through (instant, real animation later) | none | |
| Teardown | *(no route)* | — | — | Parent RunState when() fires |

**Signals used:**
- Loading→AnimateIn: `BoltSpawned` + `BreakerSpawned` + `CellsSpawned` + `WallsSpawned` messages (check_spawn_complete)
- Playing→AnimateOut: `NodeCleared` or `TimerExpired` messages (read by handle_* systems)
- Parent gets notified via: NodeState ceasing to exist (when RunState leaves Node) OR resource flag

## ChipSelectState (sub-state of RunState::ChipSelect)

| From | To | Trigger | Transition | Notes |
|------|----|---------|------------|-------|
| Loading | AnimateIn | Pass-through | none | |
| AnimateIn | Selecting | Pass-through | none | |
| Selecting | AnimateOut | `handle_chip_input` (player selects) OR `tick_chip_timer` (timer expires) | none | |
| AnimateOut | Teardown | Pass-through | none | |
| Teardown | *(no route)* | — | — | Parent RunState when() fires |

**Signals used:**
- Selecting→AnimateOut: User input (`handle_chip_input`) or timer (`tick_chip_timer`)

## RunEndState (sub-state of RunState::RunEnd)

| From | To | Trigger | Transition | Notes |
|------|----|---------|------------|-------|
| Loading | AnimateIn | Pass-through | none | |
| AnimateIn | Active | Pass-through | none | |
| Active | AnimateOut | `handle_run_end_input` (player confirms) | none | |
| AnimateOut | Teardown | Pass-through | none | |
| Teardown | *(no route)* | — | — | Parent RunState when() fires |

**Signals used:**
- Active→AnimateOut: User input (`handle_run_end_input`)

---

## Parent→Child Coordination Pattern

All parent-child routing uses the same pattern:

1. Child enters `Teardown` variant
2. Child's teardown system runs cleanup (`CleanupOnExit<ChildState>`)
3. Parent's `when()` condition detects child teardown
4. Parent transitions to next state, which tears down the child SubState automatically

**Detection mechanism options** (for `when()` conditions):
- `condition_changed_to(false, state_exists::<ChildState>())` — fire once when child SubState removed
- `on_message::<ChildTeardownComplete>()` — child sends explicit signal
- `resource_exists_and_changed::<ChildResult>()` — child writes result, parent polls

**Recommended:** Use `condition_changed_to(false, state_exists::<ChildState>())` for simplicity. No child cooperation needed — the Bevy state machine itself removes the SubState when the parent changes. This fires exactly once on the removal frame.

**For initial migration (plain Bevy routing):** The teardown systems in `state/routing.rs` call `next_state.set()` directly on the parent. No `when()` needed — the child owns the decision.

---

## Cleanup Mapping

| State | Cleanup Trigger | What's Cleaned | Replaces |
|-------|----------------|----------------|----------|
| NodeState::Teardown | OnEnter | `CleanupOnExit<NodeState>` — cells, extra bolts, walls, HUD | `CleanupOnNodeExit` on OnExit(GameState::Playing) |
| RunState::Teardown | OnEnter | `CleanupOnExit<RunState>` — primary breaker, primary bolt, run stats | `CleanupOnRunEnd` on OnExit(GameState::RunEnd) |
| ChipSelectState::Teardown | OnEnter | `CleanupOnExit<ChipSelectState>` — chip select UI | `ChipSelectScreen` on OnExit(GameState::ChipSelect) |
| RunEndState::Teardown | OnEnter | `CleanupOnExit<RunEndState>` — run end UI | `RunEndScreen` on OnExit(GameState::RunEnd) |
| MenuState::Main exit | OnExit | `MainMenuScreen` entities | same as current |
| MenuState::StartGame exit | OnExit | `RunSetupScreen` entities | same as current |
