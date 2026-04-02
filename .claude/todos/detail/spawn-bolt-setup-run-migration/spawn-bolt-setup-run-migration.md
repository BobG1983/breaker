# State Machine Refactor & Run Lifecycle

## Summary
Replace the flat `GameState` + `PlayingState` with a hierarchical state machine (AppState → GameState → MenuState/RunState → NodeState), move breaker+bolt spawning to a single `setup_run` system, replace pause with `Time<Virtual>::pause()`, and extract screen transitions into a reusable crate.

## Context
Originally scoped as "move spawn_bolt into setup_run." During interrogation, expanded to include:
- Breaker spawn migration (now has builder pattern)
- Full state machine redesign (flat GameState conflates run/node concerns)
- Pause refactor (`Time<Virtual>::pause()` replaces `PlayingState`)
- Transition refactor (message-driven overlay replaces dedicated transition states)
- Generic cleanup markers (`CleanupOnExit<S>`)

## Design Decisions

### State Machine Hierarchy

```
AppState
├── Loading        ← disk asset loading (RON, user settings, graphics config)
├── Game           ← everything below lives here
│   └── GameState (sub-state of AppState::Game)
│       ├── Loading    ← registry stuffing (chip registry, breaker registry, observers, etc.)
│       ├── Menu
│       │   └── MenuState (sub-state of GameState::Menu)
│       │       ├── Loading      ← (no-op for now, option to load menu assets)
│       │       ├── StartGame    ← fast-transitions to Run for now; future: pre-run setup
│       │       ├── Options      ← settings screen
│       │       ├── Meta         ← meta-progression / flux spending
│       │       └── Teardown     ← cleanup menu UIs
│       ├── Run
│       │   └── RunState (sub-state of GameState::Run)
│       │       ├── Loading      ← run-specific asset loading (if needed)
│       │       ├── Setup        ← run setup screen (seed entry, breaker select, mutators); OnExit spawns breaker+bolt via setup_run
│       │       ├── Node
│       │       │   └── NodeState (sub-state of RunState::Node)
│       │       │       ├── Loading      ← reset_bolt, reset_breaker, spawn cells/walls
│       │       │       ├── AnimateIn    ← cells slam onto map, level intro animation
│       │       │       ├── Playing      ← active gameplay (physics, input, collisions)
│       │       │       ├── AnimateOut   ← node-cleared celebration animation
│       │       │       └── Teardown     ← cleanup CleanupOnExit<NodeState>
│       │       ├── ChipSelect
│       │       │   └── ChipSelectState (sub-state of RunState::ChipSelect)
│       │       │       ├── Loading
│       │       │       ├── AnimateIn
│       │       │       ├── Selecting    ← chip selection UI
│       │       │       ├── AnimateOut
│       │       │       └── Teardown
│       │       ├── End          ← win/lose screen
│       │       └── Teardown     ← cleanup CleanupOnExit<RunState>
│       └── Teardown
└── Teardown       ← app shutdown
```

Every level has a mirrored Loading/Teardown pattern. Most Loading states are no-ops initially but give the option to load additional on-disk assets later using the same loading screen systems.

### Pause — `Time<Virtual>::pause()`

Pause is NOT a state. It's orthogonal — just pausing the virtual clock.

- `toggle_pause` calls `time.pause()` / `time.unpause()` on `Time<Virtual>`
- `FixedUpdate` stops entirely (no catch-up on resume) — confirmed from Bevy source
- Game timers using virtual time freeze automatically
- Input and UI on `Time<Real>` continue running
- Pause screen is a "screen" that spawns in `AppState::Game` gated on `Time<Virtual>::is_paused()`
- No `PlayingState`, no `PauseState`, no `run_if(not_paused)` on 40+ systems
- Fixes existing bugs: `bridge_node_start` and `reset_entropy_engine_on_node_start` currently fire on every unpause because they're on `OnEnter(PlayingState::Active)`

**Time model:**
- `Time<Virtual>` — game time, pausable. Default `Time` in `Update`.
- `Time<Fixed>` — fixed timestep, accumulates from `Time<Virtual>`. Default `Time` in `FixedUpdate`. Freezes when virtual paused.
- `Time<Real>` — wall clock, never paused. Used by transitions and pause screen.

### Screen Transitions — Overlay System

Transitions are NOT states. They're an overlay system owned by the `rantzsoft_lifecycle` crate.

Three internal actions the crate can execute (game never triggers these directly — routing does):
- **ChangeState** — immediate `next_state.set_if_neq(to)`, no visual effect
- **TransitionOut** — pause `Time<Virtual>`, play wipe-out overlay using `Time<Real>`, then change state (covers what's leaving)
- **TransitionIn** — change state, then play wipe-in overlay using `Time<Real>`, then unpause (reveals what's arriving)

Overlay uses `GlobalZIndex(i32::MAX - 1)` to render above everything. Effect (fade, swipe, etc.) is selected per route or randomly from a registered set.

**Pause during transition:** Virtual time is already paused → pause is a no-op. Gate pause screen on "virtual time is paused AND no active transition."

### Entity Lifecycle

- `setup_run` (new system in `run` domain): Runs on `OnExit(RunState::Setup)`. Spawns primary breaker + primary bolt via builders with `CleanupOnExit<RunState>`. Must run after Setup because the user selects their breaker on that screen.
- `reset_bolt`: Runs on `OnEnter(NodeState::Loading)`. Repositions bolt, sends `BoltSpawned`.
- `reset_breaker`: Runs on `OnEnter(NodeState::Loading)`. Repositions breaker, sends `BreakerSpawned`.
- `spawn_bolt` system: Deleted entirely.
- `spawn_or_reuse_breaker` system: Deleted entirely.

### Generic Cleanup — `CleanupOnExit<S>`

Replace `CleanupOnNodeExit` and `CleanupOnRunEnd` with `CleanupOnExit<S>` where `S` is a state type.

- `CleanupOnExit<NodeState>` — despawned on `OnEnter(NodeState::Teardown)` (cells, extra bolts, walls)
- `CleanupOnExit<RunState>` — despawned on `OnEnter(RunState::Teardown)` (primary breaker, primary bolt)
- `CleanupOnExit<MenuState>` — despawned on `OnEnter(MenuState::Teardown)` (menu UIs)
- UI style: likely SWF/SVG-style UIs with custom materials and shaders for visual identity, rather than diegetic (mixed styles may look weird). Decision not final — revisit during Phase 5 screen work.

### SubStates Nesting — Verified

4-level nesting (AppState → GameState → RunState → NodeState) is fully supported in Bevy 0.18. No depth limit. Cascade teardown works correctly (innermost exits first). Registration must be in parent-first order. See [research/substates-nesting-depth.md](research/substates-nesting-depth.md).

### Declarative Routing

The game defines ALL state transitions at setup time via a routing table. The game never calls `next_state.set()` directly. The only message the game sends is `PhaseComplete<S>` ("I'm done with this phase"). The crate looks up the route and handles everything — state change, transitions, cross-level routing.

**Route builder API (built during plugin setup):**

Destination and transition are two independent axes, each can be static or dynamic:

```rust
app
// Static destination, no transition
.add_route(Route::from(NodeState::AnimateIn)
    .to(NodeState::Active)
    .build())

// Static destination, static transition
.add_route(Route::from(NodeState::AnimateOut)
    .to(NodeState::Teardown)
    .with_transition(TransitionOut(TransitionEffect::Fade))
    .build())

// Dynamic destination, static transition (cross-level!)
.add_route(Route::from(NodeState::Teardown)
    .to_dynamic(|world| {
        match *world.resource::<NodeResult>() {
            NodeResult::Win if !is_final_node(world) => RunState::ChipSelect,
            _ => RunState::End,
        }
    })
    .with_transition(TransitionIn(TransitionEffect::Random))
    .build())

// Static destination, dynamic transition
.add_route(Route::from(MenuScreen::MainMenu)
    .to(MenuScreen::Options)
    .with_dynamic_transition(|world| {
        // pick effect based on context
        TransitionEffect::SlideLeft
    })
    .build())

// Dynamic destination, dynamic transition
.add_route(Route::from(SomeState::Foo)
    .to_dynamic(|world| { /* returns S */ })
    .with_dynamic_transition(|world| { /* returns TransitionEffect */ })
    .build())
```

**Four combinations from two independent axes:**

| | Static destination | Dynamic destination |
|---|---|---|
| **No/static transition** | `.to(S)` | `.to_dynamic(fn → S)` |
| **Dynamic transition** | `.to(S).with_dynamic_transition(fn)` | `.to_dynamic(fn → S).with_dynamic_transition(fn)` |

Builder makes invalid combos unrepresentable (can't call both `.to()` and `.to_dynamic()`). Dynamic destination functions return `S` (a state value), not a `RouteAction`. Dynamic transition functions return `TransitionEffect`. Composition replaces the need for a combined return type.

**Routing table is a `Resource` — mutable at runtime:**

Routes can be added at plugin build time (convenience) or at runtime (flexibility):

```rust
// Plugin build time — prefills the routing table
app.add_plugin(LifecyclePlugin::<NodeState>::new()
    .route(Route::from(NodeState::AnimateIn).to(NodeState::Active).build())
    .route(Route::from(NodeState::Teardown)
        .to_dynamic(node_teardown_router)
        .with_transition(TransitionIn(Random))
        .build())
);

// Runtime — game adds/removes routes dynamically
fn enable_mutator_routes(mut table: ResMut<RoutingTable<RunState>>) {
    table.add(Route::from(RunState::Setup)
        .to(RunState::MutatorSelect)
        .with_transition(TransitionIn(SlideLeft))
        .build());
}
```

This enables mutators, modifiers, or debug tools that inject extra states into the flow at runtime. The crate doesn't care — it just reads the table when `PhaseComplete` arrives.

**How it works:**
1. Game system sends `PhaseComplete<S>` ("I'm done")
2. Crate's routing system runs — exclusive system, gated by `run_if(on_message::<PhaseComplete<S>>())` so zero per-frame cost when idle
3. Uses `resource_scope` to extract the routing table (avoids borrow conflict)
4. Looks up current state in the table
5. Resolves destination: static → known value, dynamic → calls `route_fn(&World)` which **returns** a state value (never sets it)
6. Resolves transition: none → ChangeState, static → known effect, dynamic → calls `transition_fn(&World)` which **returns** a `TransitionEffect`
7. Crate executes the state change and triggers the overlay if needed — game never touches `NextState`

**Key properties:**
- Exclusive system but gated by `on_message` — zero per-frame cost when idle
- Dynamic functions are `fn(&World) -> S` or `fn(&World) -> TransitionEffect` — read-only, return values, never set state
- Game crate: pure decisions ("given this World, go here"). Crate: execution ("ok, I'll do it")
- Multiple messages same frame: process first, warn on duplicates (`NextState` has one slot)
- Cross-hierarchy cascades are naturally frame-separated (OnEnter runs next frame)
- Cross-level routing works because dynamic functions have `&World` access and can return any state type
- Zero game knowledge in the crate — crate stores function pointers, not game types
- See [research/routing-without-exclusive-world.md](research/routing-without-exclusive-world.md) and [research/declarative-routing.md](research/declarative-routing.md)

**Timing:** AnimateOut uses `Time<Virtual>` (gameplay animation). TransitionOut/TransitionIn use `Time<Real>` (screen wipe). These are sequential: AnimateOut → ChangeState to Teardown → cleanup → PhaseComplete → routing → TransitionIn to next screen.

### Messages

**Domain signals (existing pattern, updated semantics):**
- `reset_bolt` sends `BoltSpawned` on every node entry (semantics: "bolt is ready")
- `reset_breaker` sends `BreakerSpawned` on every node entry (semantics: "breaker is ready")
- `check_spawn_complete` continues to use all 4 signals (Bolt, Breaker, Cells, Walls)

**Lifecycle message — `PhaseComplete<S>`:**

The only lifecycle message the game sends. Generic over the state type. Means "I'm done with the current phase — route me." The crate handles everything else via the routing table.

**`ScreenLifecycle` trait (for states with the 5-phase pattern):**
```rust
pub trait ScreenLifecycle: States {
    fn loading()     -> Self;
    fn animate_in()  -> Self;
    fn active()      -> Self;  // Playing, Selecting, Setup, etc.
    fn animate_out() -> Self;
    fn teardown()    -> Self;
}
```

Not all states implement this — only those with the standard 5-phase lifecycle (NodeState, ChipSelectState). States like MenuState and RunState have game-specific variants and define their routes directly. Derive macro deferred until 6+ impls. See [research/enum-trait-constraints.md](research/enum-trait-constraints.md).

## Scope

### In
- New state types: `AppState`, `GameState` (revised), `MenuState`, `RunState`, `NodeState`, `ChipSelectState`
- `CleanupOnExit<S>` generic cleanup marker
- `rantzsoft_lifecycle` crate — generic screen lifecycle (phases, `PhaseComplete<S>` messages, advancing, wipe transitions, `CleanupOnExit<S>`)
- `setup_run` system spawning primary breaker + bolt
- Pause via `Time<Virtual>::pause()` + pause screen
- Rename `src/screen/` → `src/states/` (screens are state lifecycle implementations, not just UI)
- Dissolve `src/ui/` — each state owns its own UI (HUD lives in node state, chip cards in chip select state, etc.)
- Migrate ALL existing systems from old states to new states
- Delete `spawn_bolt`, `spawn_or_reuse_breaker`, `PlayingState`
- Remove all `run_if(in_state(PlayingState::Active))` guards (40+ systems)
- Update cleanup systems to use `CleanupOnExit<S>` + new teardown states
- Update architecture docs (see below)

### Out
- Effect-spawned bolts (use builder directly)
- Wall/cell builder patterns (separate todos)
- Actual menu screen implementations (just the state infrastructure)
- Actual AnimateIn/AnimateOut animations (just the states)

## Architecture Docs

### New docs (create after implementation, once patterns are concrete)

`docs/architecture/state/` directory:
- `state/index.md` — state hierarchy overview (AppState → GameState → RunState → NodeState etc.)
- `state/lifecycle.md` — the universal screen lifecycle pattern (Loading → AnimateIn → Active → AnimateOut → Teardown), phase completion messages, advancing, routing
- `state/transitions.md` — screen wipe transition system (pause virtual time, overlay, advance state)
- `state/pause.md` — `Time<Virtual>::pause()` approach, interaction with transitions
- `state/cleanup.md` — `CleanupOnExit<S>` pattern
- `state/adding-a-screen.md` — how to add a new screen/state (implement the lifecycle trait, register sub-state, add router entry)

### Existing docs needing update

These reference `GameState`, `PlayingState`, or state-related patterns:
- `docs/architecture/state.md` — replace with `state/index.md` (or redirect)
- `docs/architecture/plugins.md` — plugin registration references states
- `docs/architecture/ordering.md` — system ordering references state schedules
- `docs/architecture/data.md` — component/resource patterns
- `docs/architecture/standards.md` — code standards may reference old states
- `docs/architecture/layout.md` — layout loading may reference Playing
- `docs/architecture/effects/trigger_systems.md` — effect triggers reference Playing
- `docs/architecture/effects/reversal.md` — reversal may reference state
- `docs/architecture/bolt-definitions.md` — bolt spawn references
- `docs/architecture/builders/bolt.md` — builder docs reference spawn lifecycle
- `docs/architecture/builders/breaker.md` — builder docs reference spawn lifecycle

## Dependencies
- Depends on: Bolt builder (done), Breaker builder (done)
- Blocks: Wall builder pattern, Cell builder pattern, Bolt birthing animation, all Phase 5+ work

## Research
- [Bevy pause patterns](research/bevy-pause-patterns.md) — `Time<Virtual>::pause()` confirmed as best approach
- [Current pause implementation](research/current-pause-implementation.md) — 40+ systems gated, two bugs found
- [Bevy transition patterns](research/bevy-transition-patterns.md) — no crate needed, overlay + `GlobalZIndex` pattern
- [SubStates nesting depth](research/substates-nesting-depth.md) — 4-level nesting confirmed, no depth limit
- [Enum trait constraints](research/enum-trait-constraints.md) — associated methods trait, derive macro deferred until 6+ impls
- [Declarative routing](research/declarative-routing.md) — one-shot systems + resource_scope for dynamic routes, static routes via OnExit
- [Routing without exclusive World](research/routing-without-exclusive-world.md) — Commands::run_system, on_message run condition, multiple message handling

## Sub-Items

This should be split into ordered work items:

1. **State machine types** — Define AppState, GameState, MenuState, RunState, NodeState, ChipSelectState in shared domain
2. **`rantzsoft_lifecycle` crate** — Routing table (static + dynamic routes), `PhaseComplete<S>` message, exclusive routing system, screen wipe overlay (TransitionOut/TransitionIn), `CleanupOnExit<S>`, `ScreenLifecycle` trait
3. **Wire routing tables** — Register all routes for all states during plugin build (static routes + dynamic routes with game-specific logic)
4. **Pause refactor** — Replace PlayingState with `Time<Virtual>::pause()`, pause screen gated on `is_paused() AND no active transition`
5. **`setup_run` system** — Spawn breaker+bolt on OnExit(RunState::Setup), delete old spawn systems
6. **Migrate OnEnter/OnExit systems** — All domains move to new states, send `PhaseComplete<S>` instead of managing transitions directly
7. **Migrate FixedUpdate run_if guards** — Replace `PlayingState::Active` with `NodeState::Playing`
8. **Architecture docs update** — Create `docs/architecture/state/` directory, rewrite state.md, update all referencing docs

## Status
`ready`
