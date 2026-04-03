# State Lifecycle Refactor

## Summary
Replace the flat `GameState` + `PlayingState` with a hierarchical state machine (AppState → GameState → MenuState/RunState → NodeState), move breaker+bolt spawning to a single `setup_run` system, replace pause with `Time<Virtual>::pause()`, and build a reusable lifecycle crate with declarative routing and transitions.

## Context
Originally scoped as "move spawn_bolt into setup_run." During interrogation, expanded to include:
- Breaker spawn migration (now has builder pattern)
- Full state machine redesign (flat GameState conflates run/node concerns)
- Pause refactor (`Time<Virtual>::pause()` replaces `PlayingState`)
- Transition refactor (message-driven overlay replaces dedicated transition states)
- Generic cleanup markers (`CleanupOnExit<S>`)
- Declarative routing crate (`rantzsoft_lifecycle`)

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
│       │       ├── Main         ← main menu screen (Start, Options, Meta buttons)
│       │       ├── StartGame    ← run config screen; fast-transitions to Run for now
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
│       │       ├── RunEnd
│       │       │   └── RunEndState (sub-state of RunState::RunEnd)
│       │       │       ├── Loading
│       │       │       ├── AnimateIn
│       │       │       ├── Active       ← win/lose screen, stats, continue/quit
│       │       │       ├── AnimateOut
│       │       │       └── Teardown
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

Transitions are NOT states. They're an overlay system owned by the `rantzsoft_lifecycle` crate. The crate pauses `Time<Virtual>` during transitions, plays overlay animations using `Time<Real>`, then unpauses. Overlay uses `GlobalZIndex(i32::MAX - 1)`.

**Transition types:**
```rust
Transition::Out(effect)              // pause → wipe covers screen → change state (stays covered)
Transition::In(effect)               // wipe reveals screen → unpause
Transition::OutIn { in_e, out_e }    // pause → wipe out → change state → wipe in → unpause
Transition::OneShot(effect)          // for effects where both states coexist (e.g., Slide)
```

- **Out**: pauses virtual time, covers the screen, changes state. Screen stays covered until a matching In.
- **In**: reveals the screen, unpauses virtual time. Pairs with a preceding Out.
- **OutIn**: full cycle — pause, cover, change state, reveal, unpause. Convenience for paired Out+In.
- **OneShot**: both old and new state content coexist during the effect (e.g., slide between two already-rendered UIs). Not all effects support OneShot — `impl OneShotEffect` vs `impl TransitionEffect` trait bounds enforce this at compile time.

**Effect traits — pure markers, no methods:**

```rust
pub trait Transition: 'static + Send + Sync {}  // base marker
pub trait InTransition: Transition {}            // can reveal a screen
pub trait OutTransition: Transition {}           // can cover a screen
pub trait OneShotTransition: Transition {}       // both screens coexist
```

**Built-in transitions:**

All transitions take an `EasingCurve` and `Duration` alongside their effect-specific params. Colors use Bevy's `Color` directly.

```rust
/// Common to every transition. Defaults: 0.3s, EaseOutCubic.
#[derive(Clone)]
pub struct TransitionConfig {
    pub duration: Duration,       // default: 300ms
    pub easing: EasingCurve<f32>, // default: EaseOutCubic
}
// impl Default — all transition structs also derive Default
// (Color defaults to BLACK, directions/origins to sensible values)
```

**Built-in transitions:**

Each effect has In (reveals), Out (covers), and OutIn (composes Out → state change → In) variants. All implement the corresponding marker traits.

```rust
// ── Fade ─────────────────────────────────────────────
// Uniform alpha overlay.
pub struct FadeIn  { pub color: Color, pub config: TransitionConfig }  // In
pub struct FadeOut { pub color: Color, pub config: TransitionConfig }  // Out
pub struct FadeOutIn { pub color: Color, pub config: TransitionConfig } // In, Out

// ── Dissolve ─────────────────────────────────────────
// Procedural noise-based per-pixel cover/reveal.
pub struct DissolveIn  { pub color: Color, pub config: TransitionConfig }  // In
pub struct DissolveOut { pub color: Color, pub config: TransitionConfig }  // Out
pub struct DissolveOutIn { pub color: Color, pub config: TransitionConfig } // In, Out

// ── Pixelate ─────────────────────────────────────────
// Resolution reduction to/from solid block.
pub struct PixelateIn  { pub color: Color, pub config: TransitionConfig }  // In
pub struct PixelateOut { pub color: Color, pub config: TransitionConfig }  // Out
pub struct PixelateOutIn { pub color: Color, pub config: TransitionConfig } // In, Out

// ── Wipe ─────────────────────────────────────────────
// Solid bar sweeps across screen in a direction.
pub struct WipeIn  { pub direction: WipeDirection, pub color: Color, pub config: TransitionConfig }  // In
pub struct WipeOut { pub direction: WipeDirection, pub color: Color, pub config: TransitionConfig }  // Out
pub struct WipeOutIn { pub direction: WipeDirection, pub color: Color, pub config: TransitionConfig } // In, Out

// ── Iris ─────────────────────────────────────────────
// Circle expands/shrinks from a point.
pub struct IrisIn  { pub origin: IrisOrigin, pub color: Color, pub config: TransitionConfig }  // In
pub struct IrisOut { pub origin: IrisOrigin, pub color: Color, pub config: TransitionConfig }  // Out
pub struct IrisOutIn { pub origin: IrisOrigin, pub color: Color, pub config: TransitionConfig } // In, Out

// ── Slide ────────────────────────────────────────────
// Camera translation. OneShot only — both screens must coexist.
// Game's responsibility to ensure content is loaded and positioned.
// No color (no overlay).
pub struct Slide { pub direction: SlideDirection, pub config: TransitionConfig }  // OneShot

// ── Supporting enums ─────────────────────────────────
pub enum SlideDirection { Left, Right, Up, Down }
pub enum WipeDirection { Left, Right, Up, Down }
pub enum IrisOrigin {
    Center,
    Position(Vec2),
}
```

Each OutIn variant is sugar — internally it composes the matching Out and In effects (splitting `config.duration` in half for each phase). The routing table's `Transition::OutIn { out_e, in_e }` can also mix different effect types (e.g., `WipeOut` + `FadeIn`).

**Transition type enum — each variant boxes the right trait:**

```rust
enum TransitionType {
    Out(Box<dyn OutTransition>),
    In(Box<dyn InTransition>),
    OutIn { in_e: Box<dyn InTransition>, out_e: Box<dyn OutTransition> },
    OneShot(Box<dyn OneShotTransition>),
}
```

- Compile-time enforcement: `TransitionType::OneShot(FadeIn)` won't compile
- `OutIn` is composition — mix and match (`FadeOut` + `FadeIn`, `Slide` + `FadeIn`, etc.)
- Routing table is homogeneous — `TransitionType` is one concrete type

**Transitions are Bevy systems, not trait methods:**

Each transition effect is implemented as three normal Bevy systems (start, run, end) that react to marker resources. The crate orchestrates via resource insertion/removal and internal messages.

```rust
// Crate provides marker resources (generic over T: Transition):
StartingTransition<T>   // "set up your overlay"
RunningTransition<T>    // "animate each frame"
EndingTransition<T>     // "clean up"

// Crate-internal messages (transition systems → crate):
TransitionReady         // starting → running
TransitionRunComplete   // running → ending
TransitionOver          // ending → done
```

**Crate-internal flow:**
```
Route fires with Transition::Out(FadeOut)
  → crate looks up TypeId::of::<FadeOut>() in TransitionRegistry
  → registry closure inserts StartingTransition::<FadeOut>
  → fade_out_start system runs (spawns overlay), sends TransitionReady
  → crate removes Starting, inserts RunningTransition::<FadeOut>
  → fade_out_run system runs each frame (animates), sends TransitionRunComplete when done
  → crate removes Running, inserts EndingTransition::<FadeOut>
  → fade_out_end system runs (despawns overlay), sends TransitionOver
  → crate removes Ending, sends TransitionEnd<S> to game
```

**Registration:**

All built-in transitions (Fade*, Dissolve*, Pixelate*, Wipe*, Iris*, Slide) are registered automatically by the plugin. Custom transitions use the plugin builder:

```rust
app.add_plugin(LifecyclePlugin::new()
    // Built-ins registered automatically
    .register_custom_transition::<MyCustomWipe>(
        custom_start, custom_run, custom_end
    )
);
```

- **Traits**: `pub` — game can implement for custom effects
- **`register_transition`**: `pub(crate)` — internal plumbing
- **`.register_custom_transition::<T>()`**: `pub` — game's entry point for custom effects
- **TransitionRegistry**: maps `TypeId` → starter closure, bridges type-erased routing table → concrete resource insertion
- **Validation**: if a route references a transition not in the registry, caught at startup (scan all routes against registry)

**Pause during transition:** Virtual time is already paused → game pause is a no-op. Gate pause screen on "virtual time is paused AND no active transition."

**Deferred ChangeState during transitions:** The crate defers `ChangeState<S>` processing while a transition is active. This ensures AnimateIn plays AFTER a transition reveals (Loading completes behind the cover, route to AnimateIn queues, transition reveals, then AnimateIn fires).

### Declarative Routing

The game defines ALL state transitions at setup time via a routing table. The game never calls `next_state.set()` directly. The only message the game sends is `PhaseComplete<S>` ("I'm done with this phase"). The crate looks up the route and handles everything — state change, transitions, cross-level routing.

**Route builder API:**

Destination and transition are two independent axes, each can be static or dynamic. No `.build()` needed — `add_route` accepts the builder directly.

```rust
// Static destination, no transition (ChangeState)
app.add_route(Route::from(NodeState::AnimateIn).to(NodeState::Playing))

// Static destination, static transition
app.add_route(Route::from(NodeState::AnimateOut)
    .to(NodeState::Teardown)
    .with_transition(Transition::Out(FadeOut)))

// Dynamic destination, static transition
app.add_route(Route::from(NodeState::Teardown)
    .to_dynamic(|world| { /* returns a state value */ })
    .with_transition(Transition::In(FadeIn)))

// Static destination, dynamic transition
app.add_route(Route::from(MenuState::Main)
    .to(MenuState::Options)
    .with_dynamic_transition(|world| { /* returns a Transition */ }))

// Dynamic destination, dynamic transition
app.add_route(Route::from(SomeState::X)
    .to_dynamic(|world| { /* returns S */ })
    .with_dynamic_transition(|world| { /* returns a Transition */ }))
```

**Three independent axes:**

| Axis | Default | Override |
|------|---------|----------|
| **Destination** | (required) | `.to(S)` or `.to_dynamic(fn → S)` |
| **Transition** | ChangeState (no visual) | `.with_transition(T)` or `.with_dynamic_transition(fn)` |
| **Trigger** | `ChangeState<S>` message | `.when(fn(&World) -> bool)` |

Builder makes invalid combos unrepresentable (can't call both `.to()` and `.to_dynamic()`).

**Routing table is a `Resource` — mutable at runtime:**

Game builds via builder, validates with `build()`, registers with `add_routing_table`:

```rust
// Plugin build time — builder → build → register
let table = RoutingTable::add_route(Route::from(NodeState::Loading).to(NodeState::AnimateIn))
    .add_route(Route::from(NodeState::AnimateIn).to(NodeState::Playing))
    .build()?;
app.add_routing_table(table);

// Runtime — game mutates routes via ResMut
fn enable_mutator_routes(mut table: ResMut<RoutingTable<RunState>>) {
    table.add_route(Route::from(RunState::Setup)
        .to(RunState::MutatorSelect)
        .with_transition(Transition::OutIn { in_e: SlideLeft, out_e: SlideRight }))
        .expect("mutator route should not duplicate");
}

fn disable_mutator_routes(mut table: ResMut<RoutingTable<RunState>>) {
    table.remove_route(&RunState::Setup);
}
```

This enables mutators, modifiers, or debug tools that inject extra states into the flow at runtime.

**Routing API — DECIDED:**

Three types: builder (chainable, no validation), table (validated resource), error.

```rust
/// Error returned when `build()` or `add_route()` finds a duplicate route.
#[derive(Debug)]
pub struct RoutingError<S: States> {
    /// The `from` state variant that already has a route registered.
    pub from: S,
}

/// Route equality: two routes are equal iff `from == other.from && to == other.to`.
/// Transition and trigger are NOT part of equality — same endpoints = same route.
impl<S: States> PartialEq for Route<S> { ... }
impl<S: States> Eq for Route<S> {}

/// The routing table. Inserted as a Resource. Created via builder,
/// mutable at runtime via `add_route()` / `remove_route()` / `replace_route()`.
#[derive(Resource)]
pub struct RoutingTable<S: States> {
    routes: Vec<Route<S>>,  // private — routing systems + mutation methods access this
}

impl<S: States> RoutingTable<S> {
    /// Runtime route insertion. Returns `Err(RoutingError)` if a route
    /// for the same `from` variant already exists.
    pub fn add_route(&mut self, route: Route<S>) -> Result<(), RoutingError<S>> { ... }

    /// Runtime route removal. Removes the route for the given `from` variant.
    /// No-op if no route exists for that variant.
    pub fn remove_route(&mut self, from: &S) { ... }

    /// Convenience: remove_route(from) then add_route(route).
    /// Useful for swapping a route's destination/transition/trigger at runtime
    /// without risking a RoutingError from the existing route.
    pub fn replace_route(&mut self, route: Route<S>) { ... }
}

/// Builder for constructing a RoutingTable. Collects routes without
/// validation — `build()` validates and produces the final table.
pub struct RoutingTableBuilder<S: States> {
    routes: Vec<Route<S>>,  // private — accumulated during chaining
}

impl<S: States> RoutingTableBuilder<S> {
    pub fn new() -> Self { ... }

    /// Add a route. Always succeeds — returns `&mut Self` for chaining.
    /// Duplicate detection is deferred to `build()`.
    pub fn add_route(&mut self, route: Route<S>) -> &mut Self { ... }

    /// Validate all routes and produce a RoutingTable.
    /// Returns `Err(RoutingError)` if any two routes share a `from` variant.
    pub fn build(self) -> Result<RoutingTable<S>, RoutingError<S>> { ... }
}
```

**Two ways to get a builder:**

```rust
// Explicit — via RoutingTable::builder()
let table = RoutingTable::builder()
    .add_route(Route::from(NodeState::Loading).to(NodeState::AnimateIn))
    .add_route(Route::from(NodeState::AnimateIn).to(NodeState::Playing))
    .build()?;

// Shorthand — via RoutingTable::add_route() (creates builder, adds first route)
let table = RoutingTable::add_route(Route::from(NodeState::Loading).to(NodeState::AnimateIn))
    .add_route(Route::from(NodeState::AnimateIn).to(NodeState::Playing))
    .build()?;
```

`RoutingTable::builder()` returns `RoutingTableBuilder<S>`. `RoutingTable::add_route(route)` is sugar that creates a builder and adds the first route in one call.

**`app.add_routing_table(table)` is infallible** — the table is already validated by `build()`. No `Result` needed.

**What `add_routing_table` does internally (all transparent to the game):**
1. Inserts the `RoutingTable<S>` as a `Resource`
2. Registers messages: `app.add_message::<ChangeState<S>>()`, `app.add_message::<StateChanged<S>>()`, `app.add_message::<TransitionStart<S>>()`, `app.add_message::<TransitionEnd<S>>()`
3. Registers the two routing systems for `S` (message-triggered + condition-triggered)
4. Registers the `CleanupOnExit<S>` cleanup system

No `add_route` on `App` directly — all route construction goes through `RoutingTableBuilder`.

**How it works — two systems per state type:**

**System 1: Message-triggered routes** (exclusive, gated by `run_if(on_message::<ChangeState<S>>())`)
1. Game sends `ChangeState<S>`
2. System fires — zero per-frame cost when idle
3. Uses `resource_scope` to extract routing table
4. Looks up route for current state
5. If route trigger is **message-triggered** → resolve destination and transition, execute
6. If route trigger is **condition-triggered** → warn ("ChangeState received for a when()-triggered route"), skip
7. Crate executes the state change and triggers overlay if needed

**System 2: Condition-triggered routes** (exclusive, runs every frame)
1. Iterates only routes marked as condition-triggered
2. For each: calls `when_fn(&World)`
3. If true → resolve destination and transition, execute
4. Skips all message-triggered routes entirely

**Both systems share:**
- `resource_scope` to extract routing table (avoids borrow conflict with `&World` for dynamic functions)
- Dynamic destination: `fn(&World) -> S` — read-only, returns value, never sets state
- Dynamic transition: `fn(&World) -> TransitionType` — read-only, returns value
- Crate executes ALL state changes — game never touches `NextState`

**Key properties:**
- One route per `from` variant, one trigger type per route — no crosstalk
- Message-triggered: zero cost when idle (gated by `on_message`)
- Condition-triggered: near-zero cost (single-digit `fn(&World) -> bool` checks per frame)
- Multiple ChangeState messages same frame: process first, warn on duplicates
- Cross-hierarchy cascades are naturally frame-separated (OnEnter runs next frame)
- All routes are same-level — parents use `when()` for child completion, no cross-level routing
- Zero game knowledge in the crate — stores function pointers, not game types
- See [research/routing-without-exclusive-world.md](research/routing-without-exclusive-world.md) and [research/declarative-routing.md](research/declarative-routing.md)

### Full Routing Table

All routes are same-level. Parents use `when()` to react to children reaching terminal states.

```rust
// ═══════════════════════════════════════════════════════
// AppState: Loading | Game | Teardown
// ═══════════════════════════════════════════════════════

app.add_routing_table(
    RoutingTable::add_route(Route::from(AppState::Loading)
            .to(AppState::Game)
            .with_transition(Transition::In(FadeIn)))
        .build()?);

// ═══════════════════════════════════════════════════════
// GameState: Loading | Menu | Run | Teardown
// (sub-state of AppState::Game, default: Loading)
//
// Uses when() to react to child teardown completion flags.
// ═══════════════════════════════════════════════════════

app.add_routing_table(
    RoutingTable::add_route(Route::from(GameState::Loading)
            .to(GameState::Menu)
            .with_transition(Transition::In(FadeIn)))
        // Menu finished → go to Run
        .add_route(Route::from(GameState::Menu)
            .to(GameState::Run)
            .with_transition(Transition::OutIn { in_e: FadeIn, out_e: FadeOut })
            .when(|world| world.resource::<MenuTeardownComplete>().0))
        // Run finished → go to Menu
        .add_route(Route::from(GameState::Run)
            .to(GameState::Menu)
            .with_transition(Transition::OutIn { in_e: FadeIn, out_e: FadeOut })
            .when(|world| world.resource::<RunTeardownComplete>().0))
        .build()?);

// ═══════════════════════════════════════════════════════
// MenuState: Loading | Main | StartGame | Options | Meta | Teardown
// (sub-state of GameState::Menu, default: Loading)
//
// StartGame routes to Teardown when done. GameState's when()
// detects Teardown and transitions Menu → Run.
// ═══════════════════════════════════════════════════════

app.add_routing_table(
    RoutingTable::add_route(Route::from(MenuState::Loading)
            .to(MenuState::Main))
        .add_route(Route::from(MenuState::Main)
            .to_dynamic(|world| {
                match *world.resource::<MenuNav>() {
                    MenuNav::Start   => MenuState::StartGame,
                    MenuNav::Options => MenuState::Options,
                    MenuNav::Meta    => MenuState::Meta,
                }
            })
            .with_transition(Transition::OneShot(SlideLeft)))
        .add_route(Route::from(MenuState::StartGame)
            .to(MenuState::Teardown))                     // parent when() handles the rest
        .add_route(Route::from(MenuState::Options)
            .to(MenuState::Main)
            .with_transition(Transition::OneShot(SlideRight)))
        .add_route(Route::from(MenuState::Meta)
            .to(MenuState::Main)
            .with_transition(Transition::OneShot(SlideRight)))
        // MenuState::Teardown has no route — parent (GameState) when() fires
        .build()?);

// ═══════════════════════════════════════════════════════
// RunState: Loading | Setup | Node | ChipSelect | RunEnd | Teardown
// (sub-state of GameState::Run, default: Loading)
//
// Uses when() to react to child completion signals
// (resource flags, messages, game state).
// ═══════════════════════════════════════════════════════

app.add_routing_table(
    RoutingTable::add_route(Route::from(RunState::Loading)
            .to(RunState::Setup))
        .add_route(Route::from(RunState::Setup)            // OnExit spawns breaker+bolt
            .to(RunState::Node)
            .with_transition(Transition::In(FadeIn)))
        // Node finished → ChipSelect or RunEnd depending on result
        .add_route(Route::from(RunState::Node)
            .to_dynamic(|world| {
                match *world.resource::<NodeResult>() {
                    NodeResult::Win if !is_final_node(world) => RunState::ChipSelect,
                    _ => RunState::RunEnd,
                }
            })
            .with_transition(Transition::OutIn { in_e: FadeIn, out_e: FadeOut })
            .when(|world| world.resource::<NodeTeardownComplete>().0))
        // ChipSelect finished → next Node
        .add_route(Route::from(RunState::ChipSelect)
            .to(RunState::Node)
            .with_transition(Transition::OutIn { in_e: FadeIn, out_e: FadeOut })
            .when(|world| world.resource::<Messages<ChipSelectDone>>()
                .iter_current_update_messages().count() > 0))
        // RunEnd finished → Teardown (parent GameState when() handles the rest)
        .add_route(Route::from(RunState::RunEnd)
            .to(RunState::Teardown)
            .when(|world| world.resource::<PlayerLives>().0 == 0
                || world.resource::<RunEndAcknowledged>().0))
        // RunState::Teardown has no route — parent (GameState) when() fires
        .build()?);

// ═══════════════════════════════════════════════════════
// NodeState: Loading | AnimateIn | Playing | AnimateOut | Teardown
// (sub-state of RunState::Node, default: Loading)
//
// All same-level ChangeState. No transitions within the node.
// AnimateIn plays AFTER the transition that brought us here
// (crate defers ChangeState during active transitions).
// AnimateOut plays BEFORE the parent transition takes us away.
// NodeState::Teardown has no route — parent (RunState) when() fires.
// ═══════════════════════════════════════════════════════

app.add_routing_table(
    RoutingTable::add_route(Route::from(NodeState::Loading)
            .to(NodeState::AnimateIn))
        .add_route(Route::from(NodeState::AnimateIn)
            .to(NodeState::Playing))
        .add_route(Route::from(NodeState::Playing)
            .to(NodeState::AnimateOut))
        .add_route(Route::from(NodeState::AnimateOut)
            .to(NodeState::Teardown))
        // NodeState::Teardown has no route — parent (RunState) when() fires
        .build()?);

// ═══════════════════════════════════════════════════════
// ChipSelectState: Loading | AnimateIn | Selecting | AnimateOut | Teardown
// (sub-state of RunState::ChipSelect, default: Loading)
// ═══════════════════════════════════════════════════════

app.add_routing_table(
    RoutingTable::add_route(Route::from(ChipSelectState::Loading)
            .to(ChipSelectState::AnimateIn))
        .add_route(Route::from(ChipSelectState::AnimateIn)
            .to(ChipSelectState::Selecting))
        .add_route(Route::from(ChipSelectState::Selecting)
            .to(ChipSelectState::AnimateOut))
        .add_route(Route::from(ChipSelectState::AnimateOut)
            .to(ChipSelectState::Teardown))
        // ChipSelectState::Teardown has no route — parent (RunState) when() fires
        .build()?);

// ═══════════════════════════════════════════════════════
// RunEndState: Loading | AnimateIn | Active | AnimateOut | Teardown
// (sub-state of RunState::RunEnd, default: Loading)
// ═══════════════════════════════════════════════════════

app.add_routing_table(
    RoutingTable::add_route(Route::from(RunEndState::Loading)
            .to(RunEndState::AnimateIn))
        .add_route(Route::from(RunEndState::AnimateIn)
            .to(RunEndState::Active))
        .add_route(Route::from(RunEndState::Active)
            .to(RunEndState::AnimateOut))
        .add_route(Route::from(RunEndState::AnimateOut)
            .to(RunEndState::Teardown))
        // RunEndState::Teardown has no route — parent (RunState) when() fires
        .build()?);
```

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

### Messages

**Domain signals (existing pattern, updated semantics):**
- `reset_bolt` sends `BoltSpawned` on every node entry (semantics: "bolt is ready")
- `reset_breaker` sends `BreakerSpawned` on every node entry (semantics: "breaker is ready")
- `check_spawn_complete` coordinates domain signals during NodeState::Loading, sends `ChangeState<NodeState>` when all domains ready

**Messages:**

| Message | Direction | When |
|---------|-----------|------|
| `ChangeState<S>` | game → crate | "Route me from my current state" |
| `StateChanged<S> { from, to }` | crate → game | After every state change |
| `TransitionStart<S> { from, to }` | crate → game | Transition animation beginning (virtual time pausing) |
| `TransitionEnd<S> { from, to }` | crate → game | Transition animation finished (virtual time resuming) |

For OutIn: `TransitionStart → Out animation → state change → In animation → TransitionEnd`
For In/Out: `TransitionStart → animation → TransitionEnd`
For OneShot: `TransitionStart → animation → TransitionEnd`
No transition (ChangeState): only `StateChanged`, no TransitionStart/End.

`when()` routes need no messages — the crate polls conditions and fires routes directly.

Game can listen for TransitionStart/End to coordinate audio, loading indicators, analytics, particle systems, etc.

**No `ScreenLifecycle` trait.** The routing table IS the lifecycle definition. States can have whatever variants they want — the crate prescribes nothing about variant names or structure. The crate provides utilities (`RoutingTable<S>`, `ChangeState<S>`, `StateChanged<S>`, `CleanupOnExit<S>`, transition overlay, `when()` polling) but no required trait implementation.

## Scope

### In
- New state types: `AppState`, `GameState` (revised), `MenuState`, `RunState`, `NodeState`, `ChipSelectState`, `RunEndState`
- `CleanupOnExit<S>` generic cleanup marker
- `rantzsoft_lifecycle` crate — declarative routing table, `ChangeState<S>` / `StateChanged<S>` / `TransitionStart<S>` / `TransitionEnd<S>` messages, exclusive routing system with `on_message` gate, `when()` polling system, transition overlay (Out/In/OutIn/OneShot) as Bevy systems with marker resources + registry, effect marker traits, `CleanupOnExit<S>`
- `setup_run` system spawning primary breaker + bolt
- Pause via `Time<Virtual>::pause()` + pause screen
- Rename `src/screen/` → `src/states/` (screens are state lifecycle implementations, not just UI)
- Dissolve `src/ui/` — each state owns its own UI (HUD lives in node state, chip cards in chip select state, etc.)
- Migrate ALL existing systems from old states to new states
- Delete `spawn_bolt`, `spawn_or_reuse_breaker`, `PlayingState`
- Remove all `run_if(in_state(PlayingState::Active))` guards (40+ systems)
- Update cleanup systems to use `CleanupOnExit<S>` + new teardown states
- Update architecture docs (see below)

### In (absorbed from Phase 5p)
- Built-in transition effects: Fade (In/Out/OutIn), Dissolve (In/Out/OutIn), Pixelate (In/Out/OutIn), Wipe (In/Out/OutIn), Iris (In/Out/OutIn), Slide
- Transition overlay rendering (GlobalZIndex, Time<Real> animation)
- Custom transition registration via `.register_custom_transition::<T>()`

### Out
- Effect-spawned bolts (use builder directly)
- Wall/cell builder patterns (separate todos)
- Actual menu screen implementations (just the state infrastructure)
- Actual AnimateIn/AnimateOut animations (just the states, transitions cover the screen-level wipes)

## Decided (implementation details)
- **No cross-level routing** — All routes same-level. Parents use `when()`. Routing table updated.
- **Out/In pairing** — User's responsibility, not enforced by crate.
- **`RunState::RunEnd`** — Added RunEndState sub-state. State tree and routing table updated.
- **No `ScreenLifecycle` trait** — Removed. Routing table IS the lifecycle. No required variant names.
- **Transition effect traits** — Three independent marker traits (`InTransition`, `OutTransition`, `OneShotTransition`). No methods. `OutIn` is composition. Each `TransitionType` variant boxes the right trait.
- **Transition implementation** — Bevy systems (start/run/end) + marker resources + TypeId registry. Custom via `.register_custom_transition::<T>()`.
- **Duplicate route handling** — `RoutingTableBuilder<S>` collects routes via chainable `add_route()` (returns `&mut Self`, always succeeds). `build()` validates (checks dup `from` variants) and returns `Result<RoutingTable<S>, RoutingError<S>>`. `app.add_routing_table(table)` is infallible — table is pre-validated. See struct definitions in the Declarative Routing section.
- **`when()` conditional routing** — `when(fn(&World) -> bool)` polled. Maximally flexible. See [research/event-driven-route-conditions.md](research/event-driven-route-conditions.md).
- **`ChangeState<S>` as generic Message** — `#[derive(Message)]` works on generic structs in Bevy 0.18. `app.add_routing_table(RoutingTable<S>)` internally calls `app.add_message::<ChangeState<S>>()`, `app.add_message::<StateChanged<S>>()`, `app.add_message::<TransitionStart<S>>()`, `app.add_message::<TransitionEnd<S>>()` during registration. Also registers the two routing systems (message-triggered + condition-triggered) for `S`. All internal — game never does manual message or system registration for these. Each `add_routing_table` call for a new `S` wires up everything the crate needs to route that state type.
- **`when()` + `ChangeState` interaction** — One route per `from` variant. Trigger type is a third axis in the builder. Default = message-triggered (`ChangeState<S>`). `.when()` overrides to condition-triggered. Mutually exclusive — can't have both for the same `from`.

## Needs Detail
(none — all design decisions resolved)

## Architecture Docs

### New docs (create after implementation, once patterns are concrete)

`docs/architecture/state/` directory:
- `state/index.md` — state hierarchy overview (AppState → GameState → RunState → NodeState etc.)
- `state/routing.md` — declarative routing (PhaseComplete, when(), route builder, routing table resource)
- `state/transitions.md` — transition types (Out/In/OutIn/OneShot), effect traits, overlay system
- `state/pause.md` — `Time<Virtual>::pause()` approach, interaction with transitions
- `state/cleanup.md` — `CleanupOnExit<S>` pattern
- `state/adding-a-screen.md` — how to add a new screen/state (define enum, register sub-state, add routes)

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
- [Routing without exclusive World](research/routing-without-exclusive-world.md) — exclusive system with on_message run condition, deferred PhaseComplete during transitions

## Implementation Detail Files

The Sub-Items have been expanded into detailed implementation files:

| File | Contents |
|------|----------|
| [implementation-waves.md](implementation-waves.md) | 8 waves with sub-waves, parallelism, branch names |
| [system-moves.md](system-moves.md) | Every system — stays, moves, or deleted, with exact paths |
| [system-changes.md](system-changes.md) | Merges, splits, rewrites — what logic changes |
| [post-restructure-tree.md](post-restructure-tree.md) | Expected src/ folder tree after Wave 2 |
| [state-assignments.md](state-assignments.md) | Every system's current state → target state |
| [routing-tables.md](routing-tables.md) | Each state's routing: from, to, trigger, transition |
| [crate-design.md](crate-design.md) | rantzsoft_lifecycle crate full specification |
| [crate-migration.md](crate-migration.md) | Systems updating for lifecycle crate (Wave 7) |
| [scenario-runner-impact.md](scenario-runner-impact.md) | Every change in breaker-scenario-runner (18 files) |
| [migration-mapping.md](migration-mapping.md) | Index file pointing to above |

## Pre-Implementation Notes
- ~~**Review implementation-waves.md before launching** — waves were written before the latest codebase changes. Revalidate wave ordering, file paths, and parallelism assumptions against current state.~~ **DONE** — validated 2026-04-02. See planning adjustments below.
- ~~**Revalidate scenario-runner-impact.md** — recently verified but should be spot-checked before the scenario runner wave.~~ **DONE** — validated 2026-04-02. Actual: 22 files (vs 18 estimated).
- **Phase 5p scope reduced** — transition visual effects absorbed into this todo. Update Phase 5p detail to reflect reduced scope.

## Planning Adjustments (2026-04-02)

Validated codebase against implementation-waves.md. Key findings:

**Discrepancies from estimates:**
- PlayingState references: 153 across 55 files (todo said "40+") — Wave 4b split into 5 batches
- Scenario runner files: 22 with GameState coupling (todo said 18) — extra 3 are test files
- All other estimates confirmed accurate

**Execution adjustments:**
- Wave 2c (generate_node_sequence refactor): SKIPPED — research concluded keep-as-is
- Wave 4/5 parallelism: REJECTED — sequential to avoid merge complexity
- All sub-waves within each wave: sequential (not parallel) to avoid lib.rs/game.rs conflicts
- Scenario runner updates: integrated per-wave, not deferred
- Shader research for transitions: launches during Wave 4, ready for Wave 5

**Wave 5 expanded (5a-5h, was 5a-5g):**
- 5f: Transition overlay infrastructure (was infrastructure + Fade)
- 5g: ALL 16 built-in effect structs (Fade, Dissolve, Pixelate, Wipe, Iris, Slide — each In/Out/OutIn + Slide OneShot)
- 5h: Plugin integration + end-to-end tests (was 5g)

**Full plan:** `.claude/plans/elegant-stirring-river.md`

## Status
`[ready]` — design complete, plan validated and approved. Blocked by todo #2 (wall-builder-pattern).

