# State Management

Bevy `States` and `SubStates` form a four-level hierarchy. State routing is declarative — each plugin registers its transitions via `rantzsoft_lifecycle::Route` entries; domain systems send `ChangeState<S>` messages when they are done and let the routing table decide the destination.

## State Hierarchy

```
AppState (top-level States)
├── Loading        ← disk asset loading (RON files, user settings); default/initial
├── Game           ← everything below lives here (iyes_progress advances Loading → Game)
│   └── GameState  (SubState of AppState::Game)
│       ├── Loading    ← registry stuffing (chip/breaker/bolt/wall registries, second-phase assets)
│       ├── Menu
│       │   └── MenuState  (SubState of GameState::Menu)
│       │       ├── Loading      ← pass-through
│       │       ├── Main         ← main menu screen (Start, Settings, Quit)
│       │       ├── StartGame    ← seed entry + breaker selection
│       │       ├── Options      ← settings screen (future)
│       │       ├── Meta         ← meta-progression / Flux spending (future)
│       │       └── Teardown     ← cleans up menu UIs; parent GameState watches for this
│       ├── Run
│       │   └── RunState  (SubState of GameState::Run)
│       │       ├── Loading      ← reset run state, generate node sequence, capture seed
│       │       ├── Setup        ← run config screen; OnExit spawns breaker + bolt
│       │       ├── Node
│       │       │   └── NodeState  (SubState of RunState::Node)
│       │       │       ├── Loading      ← spawn cells, walls, HUD; apply node scaling
│       │       │       ├── AnimateIn    ← node entrance animation (pass-through)
│       │       │       ├── Playing      ← active gameplay; physics, timers, input all active
│       │       │       ├── AnimateOut   ← node-cleared animation (pass-through)
│       │       │       └── Teardown     ← cleanup CleanupOnExit<NodeState>
│       │       ├── ChipSelect
│       │       │   └── ChipSelectState  (SubState of RunState::ChipSelect)
│       │       │       ├── Loading
│       │       │       ├── AnimateIn
│       │       │       ├── Selecting    ← player picks a chip
│       │       │       ├── AnimateOut
│       │       │       └── Teardown
│       │       ├── RunEnd
│       │       │   └── RunEndState  (SubState of RunState::RunEnd)
│       │       │       ├── Loading
│       │       │       ├── AnimateIn
│       │       │       ├── Active       ← win/lose screen, highlights, stats
│       │       │       ├── AnimateOut
│       │       │       └── Teardown
│       │       └── Teardown     ← cleanup CleanupOnExit<RunState>; parent GameState watches for this
│       └── Teardown
└── Teardown   ← app shutdown (not used in normal flow)
```

All state enum types live in `breaker-game/src/state/types/`. Each sub-state is registered by `StatePlugin` in `breaker-game/src/state/plugin.rs`.

## Declarative Routing via rantzsoft_lifecycle

State transitions use the `rantzsoft_lifecycle` crate (`RantzLifecyclePlugin`, `Route`, `RoutingTable<S>`, `ChangeState<S>`, `StateChanged<S>`). No domain calls `NextState` directly; they send a destination-less message instead.

**Route types:**

- **Message-triggered** (default) — fires when a `ChangeState<S>` message arrives while the routing table has a matching `from` state. Used when a domain system decides it is done (e.g., `handle_chip_input` → `handle_node_cleared`).
- **Condition-triggered** (`.when(fn)`) — polled each `Update` frame. Used when a parent state watches a child sub-state for teardown (e.g., `GameState::Run → GameState::Menu` fires when `RunState == Teardown`).
- **Static destination** (`.to(S)`) — hard-coded next state.
- **Dynamic destination** (`.to_dynamic(fn)`) — computed at dispatch time from world state (e.g., `RunState::Node` goes to `ChipSelect` or `RunEnd` based on `NodeOutcome`).

**Route registration** happens in `state/plugin.rs` via `register_routing()`, split into `register_parent_routes`, `register_run_routes`, `register_node_routes`, `register_chip_select_routes`, and `register_run_end_routes`.

**Lifecycle messages** (per state type `S`, registered by `RantzLifecyclePlugin::register_state::<S>()`):

- `ChangeState<S>` — sent by domain systems to request a transition from the current state
- `StateChanged<S> { from, to }` — sent by the routing system after every completed transition
- `TransitionStart<S> { from, to }` — sent before a transition effect begins
- `TransitionEnd<S> { from, to }` — sent after transition effects complete and cleanup is done

## Transition Effects

Transition effects (fade, dissolve, wipe, iris, pixelate, slide) are implemented in `rantzsoft_lifecycle` and registered on individual routes via `.with_transition(TransitionType::Out(...))`, `.with_transition(TransitionType::In(...))`, or `.with_transition(TransitionType::OutIn { out_e, in_e })`.

The lifecycle crate pauses `Time<Virtual>` during Out-type transitions and unpauses after In-type transitions complete. Overlay animations run on `Time<Real>` so they are not affected by the pause.

## Pause

Pause is NOT a state. `toggle_pause` calls `time.pause()` / `time.unpause()` on `Time<Virtual>`. `FixedUpdate` freezes entirely (no catch-up on resume). Input and UI systems that use `Time<Real>` continue running. The pause screen is spawned in `GameState::Run` gated on `Time<Virtual>::is_paused()`.

Time model:
- `Time<Virtual>` — game time, pausable. The default `Time` in `Update`.
- `Time<Fixed>` — fixed timestep, accumulates from `Time<Virtual>`. The default `Time` in `FixedUpdate`. Freezes when virtual is paused.
- `Time<Real>` — wall clock, never paused. Used by transition overlays and the pause screen.

## Entity Cleanup

`CleanupOnExit<S>` (from `rantzsoft_lifecycle`) marks entities for automatic despawn when state `S` exits. `StatePlugin` registers `cleanup_on_exit::<S>` on `OnEnter(S::Teardown)` for `NodeState`, `ChipSelectState`, `RunEndState`, and `RunState`. Entities with `CleanupOnNodeExit` or `CleanupOnRunEnd` (legacy markers, still transitioning) are handled by the `cleanup_entities` system.

## Passive types vs. active logic

State enum types and cleanup markers are passive types defined in `state/types/` (for state enums) and `shared.rs` (for legacy cleanup markers), imported by all domains. Routing declarations, transition wiring, and cleanup system registration all live in `state/plugin.rs`.
