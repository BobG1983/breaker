# Crate Design: `rantzsoft_lifecycle`

Standalone design document for the `rantzsoft_lifecycle` crate. Extracted from the
[state lifecycle refactor spec](state-lifecycle-refactor.md) and the research files under
[research/](research/). Intended as a reference that a writer-code agent can implement from.

---

## 1. Crate Purpose

`rantzsoft_lifecycle` is a Bevy 0.18 plugin crate that provides declarative state routing,
screen transitions, generic cleanup markers, and lifecycle messages for any game using Bevy's
state machine.

### What It Provides

- **Route builder API** for declaring state-to-state transitions at setup time
- **RoutingTable<S>** resource per state type, mutable at runtime
- **Two dispatch systems** per state type: message-triggered and condition-triggered
- **Transition overlay system**: Out/In/OutIn/OneShot with `Time<Virtual>` pause/unpause
- **Transition effect traits**: marker traits for compile-time enforcement of effect direction
- **TransitionRegistry**: maps `TypeId` to system triples (start/run/end) for each effect
- **Marker resources**: `StartingTransition<T>`, `RunningTransition<T>`, `EndingTransition<T>`
- **Lifecycle messages**: `ChangeState<S>`, `StateChanged<S>`, `TransitionStart<S>`, `TransitionEnd<S>`
- **CleanupOnExit<S>**: generic component + despawn system for state-scoped entity cleanup
- **RantzLifecyclePlugin**: builder-pattern plugin for registration
- **Startup validation**: routes referencing unregistered transitions are caught at app build

### What It Does NOT Contain

Per the `rantzsoft_*` zero-game-knowledge rule:

- No references to bolt, breaker, cell, node, bump, flux, or any game vocabulary
- No references to `breaker-game` types, messages, or resources
- No game-specific enums, constants, or configurations
- No opinion on state variant names -- the crate prescribes nothing about what variants a
  state enum should have (no `ScreenLifecycle` trait with required variant names)
- No cross-level routing logic -- all routes are same-level; cross-hierarchy routing is the
  game's responsibility via dynamic destination closures

---

## 2. Public API Overview

### Types

| Type | Kind | Description |
|------|------|-------------|
| `Route<S>` | Builder struct | Fluent builder for declaring a single route |
| `RoutingTable<S>` | Resource | Stores all routes for one state type |
| `TransitionType` | Enum | Out / In / OutIn / OneShot -- boxes the right trait |
| `TransitionRegistry` | Resource | Maps `TypeId` -> starter/runner/ender system triples |
| `StartingTransition<T>` | Resource (marker) | Inserted when a transition effect begins setup |
| `RunningTransition<T>` | Resource (marker) | Inserted when a transition effect is animating |
| `EndingTransition<T>` | Resource (marker) | Inserted when a transition effect is cleaning up |
| `CleanupOnExit<S>` | Component | Marks entities for despawn when state `S` enters teardown |
| `RantzLifecyclePlugin` | Plugin (builder) | Entry point for app registration |

### Traits

| Trait | Bounds | Purpose |
|-------|--------|---------|
| `Transition` | `'static + Send + Sync` | Base marker for all transition effects |
| `InTransition` | `: Transition` | Effect can reveal a screen (used by In, OutIn) |
| `OutTransition` | `: Transition` | Effect can cover a screen (used by Out, OutIn) |
| `OneShotTransition` | `: Transition` | Both screens coexist during effect |

### Messages

| Message | Direction | Payload |
|---------|-----------|---------|
| `ChangeState<S>` | game -> crate | Trigger: "route me from current state" (no payload needed) |
| `StateChanged<S>` | crate -> game | `{ from: S, to: S }` after every state change |
| `TransitionStart<S>` | crate -> game | `{ from: S, to: S }` when transition animation begins |
| `TransitionEnd<S>` | crate -> game | `{ from: S, to: S }` when transition animation finishes |

### Functions / Methods

| Item | Visibility | Signature |
|------|------------|-----------|
| `Route::from(S)` | `pub` | `fn from(state: S) -> RouteBuilder<S, NoDest, NoTransition, NoTrigger>` |
| `RouteBuilder::to` | `pub` | `fn to(self, dest: S) -> RouteBuilder<S, StaticDest, ...>` |
| `RouteBuilder::to_dynamic` | `pub` | `fn to_dynamic(self, f: fn(&World) -> S) -> RouteBuilder<S, DynamicDest, ...>` |
| `RouteBuilder::with_transition` | `pub` | `fn with_transition(self, t: TransitionType) -> RouteBuilder<S, ..., StaticTrans, ...>` |
| `RouteBuilder::with_dynamic_transition` | `pub` | `fn with_dynamic_transition(self, f: fn(&World) -> TransitionType) -> ...` |
| `RouteBuilder::when` | `pub` | `fn when(self, f: fn(&World) -> bool) -> RouteBuilder<S, ..., ..., ConditionTrigger>` |
| `RoutingTable::add` | `pub` | `fn add(&mut self, route: Route<S>) -> Result<(), DuplicateRouteError>` |
| `RantzLifecyclePlugin::new` | `pub` | `fn new() -> Self` |
| `RantzLifecyclePlugin::register_state` | `pub` | `fn register_state<S: States>(&mut self) -> &mut Self` |
| `RantzLifecyclePlugin::register_custom_transition` | `pub` | `fn register_custom_transition<T: Transition>(&mut self, start, run, end) -> &mut Self` |

---

## 3. Route Builder

The route builder uses typestate to make invalid combinations unrepresentable at compile time.

### Three Independent Axes

| Axis | Default | Override | Mutually Exclusive |
|------|---------|----------|--------------------|
| **Destination** | (required -- no default) | `.to(S)` or `.to_dynamic(fn(&World) -> S)` | Cannot call both |
| **Transition** | `ChangeState` (no visual) | `.with_transition(T)` or `.with_dynamic_transition(fn)` | Cannot call both |
| **Trigger** | Message-triggered (`ChangeState<S>`) | `.when(fn(&World) -> bool)` | Cannot call both message + condition |

### Typestate Encoding

```rust
// Marker types for typestate
pub struct NoDest;
pub struct StaticDest;
pub struct DynamicDest;

pub struct NoTransition;
pub struct StaticTransition;
pub struct DynamicTransition;

pub struct MessageTrigger;     // default
pub struct ConditionTrigger;

pub struct RouteBuilder<S, Dest, Trans, Trigger> {
    from: S,
    destination: DestinationKind<S>,
    transition: TransitionKind,
    trigger: TriggerKind,
    _phantom: PhantomData<(Dest, Trans, Trigger)>,
}
```

Methods are only available on the appropriate typestate:

- `.to()` is available on `RouteBuilder<S, NoDest, _, _>` -- returns `RouteBuilder<S, StaticDest, _, _>`
- `.to_dynamic()` is available on `RouteBuilder<S, NoDest, _, _>` -- returns `RouteBuilder<S, DynamicDest, _, _>`
- `.with_transition()` is available on `RouteBuilder<S, _, NoTransition, _>`
- `.with_dynamic_transition()` is available on `RouteBuilder<S, _, NoTransition, _>`
- `.when()` is available on `RouteBuilder<S, _, _, MessageTrigger>` -- returns `RouteBuilder<S, _, _, ConditionTrigger>`

The builder is consumed by `RoutingTable::add()`. No `.build()` call needed.

### Internal Storage

```rust
enum DestinationKind<S> {
    Static(S),
    Dynamic(Box<dyn Fn(&World) -> S + Send + Sync>),
}

enum TransitionKind {
    None,  // plain ChangeState, no visual
    Static(TransitionType),
    Dynamic(Box<dyn Fn(&World) -> TransitionType + Send + Sync>),
}

enum TriggerKind {
    Message,
    Condition(Box<dyn Fn(&World) -> bool + Send + Sync>),
}
```

### Usage Examples

```rust
// Static destination, no transition
Route::from(NodeState::AnimateIn).to(NodeState::Playing)

// Static destination, static transition
Route::from(NodeState::AnimateOut)
    .to(NodeState::Teardown)
    .with_transition(TransitionType::Out(Box::new(FadeOut)))

// Dynamic destination, static transition
Route::from(NodeState::Teardown)
    .to_dynamic(|world| { /* read resources, return S */ })
    .with_transition(TransitionType::In(Box::new(FadeIn)))

// Static destination, condition-triggered (polling)
Route::from(GameState::Menu)
    .to(GameState::Run)
    .with_transition(TransitionType::OutIn {
        in_e: Box::new(FadeIn),
        out_e: Box::new(FadeOut),
    })
    .when(|world| world.resource::<MenuTeardownComplete>().0)
```

---

## 4. RoutingTable<S>

A `Resource` storing all routes for a single state type `S`.

### Struct

```rust
#[derive(Resource)]
pub struct RoutingTable<S: States> {
    routes: HashMap<S, Route<S>>,
}
```

The key is the `from` state variant. One route per `from` variant -- no multi-route dispatch.

### `add` Returns Result

```rust
impl<S: States + Eq + Hash> RoutingTable<S> {
    pub fn add(&mut self, route: Route<S>) -> Result<(), DuplicateRouteError> {
        let from = route.from.clone();
        if self.routes.contains_key(&from) {
            return Err(DuplicateRouteError {
                state_type: std::any::type_name::<S>(),
                variant: format!("{:?}", from),
            });
        }
        self.routes.insert(from, route);
        Ok(())
    }
}

#[derive(Debug)]
pub struct DuplicateRouteError {
    pub state_type: &'static str,
    pub variant: String,
}
```

The choice of `Result` (vs. panic) allows the game to handle duplicates gracefully at runtime
when routes are added dynamically. For plugin-build-time registration, the game can `.unwrap()`
or use the `App` extension method which panics on duplicate.

### Route Key Equality

Routes are keyed by the `from` variant of `S`. Since `S: States` implies `S: Eq + Hash`,
the `HashMap<S, Route<S>>` key comparison uses the state enum's derived `Eq`/`Hash` impls.
No custom `Eq` needed -- the `from` field IS the key.

### App Extension for Convenience

```rust
pub trait RoutingTableAppExt {
    fn add_route<S: States + Eq + Hash>(
        &mut self,
        route: Route<S>,
    ) -> &mut Self;
}

impl RoutingTableAppExt for App {
    fn add_route<S: States + Eq + Hash>(
        &mut self,
        route: Route<S>,
    ) -> &mut Self {
        self.world_mut()
            .resource_mut::<RoutingTable<S>>()
            .add(route)
            .unwrap_or_else(|e| panic!("Duplicate route: {:?}", e));
        self
    }
}
```

This enables `app.add_route(Route::from(...).to(...))` chaining at plugin build time.

### Runtime Mutability

The `RoutingTable<S>` is a normal Bevy `Resource` and can be mutated at runtime via
`ResMut<RoutingTable<S>>`:

```rust
fn enable_mutator_routes(mut table: ResMut<RoutingTable<RunState>>) {
    table.add(Route::from(RunState::Setup)
        .to(RunState::MutatorSelect)
        .with_transition(TransitionType::OutIn {
            in_e: Box::new(SlideLeft),
            out_e: Box::new(SlideRight),
        }))
    .expect("route should not already exist");
}
```

---

## 5. Dispatch Systems

Two exclusive systems per registered state type handle route execution. Both use
`resource_scope` to extract the routing table, avoiding borrow conflicts with `&World`
access in dynamic closures.

### System 1: Message-Triggered Routes

Gated by `run_if(on_message::<ChangeState<S>>())` -- zero per-frame cost when idle.

```rust
fn dispatch_message_routes<S: States + Eq + Hash + Clone>(world: &mut World) {
    // Read current state
    let current = world.resource::<State<S>>().get().clone();

    // Check for active transition -- defer if one is running
    if world.contains_resource::<ActiveTransition>() {
        // Queue the ChangeState for later (see Section 6: deferred ChangeState)
        return;
    }

    world.resource_scope(|world, table: Mut<RoutingTable<S>>| {
        let Some(route) = table.routes.get(&current) else { return; };

        // Warn if route is condition-triggered
        if matches!(route.trigger, TriggerKind::Condition(_)) {
            tracing::warn!(
                "ChangeState<{}> received for condition-triggered route from {:?} -- skipping",
                std::any::type_name::<S>(),
                current,
            );
            return;
        }

        // Resolve destination
        let destination = match &route.destination {
            DestinationKind::Static(s) => s.clone(),
            DestinationKind::Dynamic(f) => f(world),
        };

        // Resolve transition
        let transition = match &route.transition {
            TransitionKind::None => None,
            TransitionKind::Static(t) => Some(t.clone()),
            TransitionKind::Dynamic(f) => Some(f(world)),
        };

        // Execute (see "Route Execution" below)
        execute_route::<S>(world, current, destination, transition);
    });
}
```

**Run condition**: `on_message::<ChangeState<S>>()` is confirmed to work in Bevy 0.18.
The condition's `MessageReader` cursor is independent of the system body's cursor (verified
in [research/routing-without-exclusive-world.md](research/routing-without-exclusive-world.md)).

**Multiple messages per frame**: Process the first, warn on duplicates. Only one route fires
per frame for a given state type.

### System 2: Condition-Triggered Routes

Runs every frame in `Update`. Iterates only routes with `TriggerKind::Condition`.

```rust
fn dispatch_condition_routes<S: States + Eq + Hash + Clone>(world: &mut World) {
    let current = world.resource::<State<S>>().get().clone();

    // Skip if transition is active
    if world.contains_resource::<ActiveTransition>() {
        return;
    }

    world.resource_scope(|world, table: Mut<RoutingTable<S>>| {
        let Some(route) = table.routes.get(&current) else { return; };

        let TriggerKind::Condition(ref when_fn) = route.trigger else { return; };

        if !when_fn(world) { return; }

        let destination = match &route.destination {
            DestinationKind::Static(s) => s.clone(),
            DestinationKind::Dynamic(f) => f(world),
        };

        let transition = match &route.transition {
            TransitionKind::None => None,
            TransitionKind::Static(t) => Some(t.clone()),
            TransitionKind::Dynamic(f) => Some(f(world)),
        };

        execute_route::<S>(world, current, destination, transition);
    });
}
```

**Performance**: Near-zero cost. Per state type, iterates at most one route (the one matching
the current state). The `when()` function is a single `fn(&World) -> bool` call. Message-
triggered routes are skipped entirely.

### Route Execution (Shared)

```rust
fn execute_route<S: States + Clone>(
    world: &mut World,
    from: S,
    to: S,
    transition: Option<TransitionType>,
) {
    match transition {
        None => {
            // Direct state change, no visual transition
            world.resource_mut::<NextState<S>>().set_if_neq(to.clone());
            // Send StateChanged message
            world.resource_mut::<Messages<StateChanged<S>>>()
                .write(StateChanged { from, to });
        }
        Some(t) => {
            // Begin transition orchestration (see Section 6)
            begin_transition::<S>(world, from, to, t);
        }
    }
}
```

---

## 6. Transition System

### Marker Traits

Pure markers with no methods. Enforce at compile time which effects can be used in which
transition direction.

```rust
pub trait Transition: 'static + Send + Sync {}
pub trait InTransition: Transition {}
pub trait OutTransition: Transition {}
pub trait OneShotTransition: Transition {}
```

Example implementations (built-in):

```rust
pub struct FadeIn;
impl Transition for FadeIn {}
impl InTransition for FadeIn {}

pub struct FadeOut;
impl Transition for FadeOut {}
impl OutTransition for FadeOut {}

pub struct Slide;
impl Transition for Slide {}
impl InTransition for Slide {}
impl OutTransition for Slide {}
impl OneShotTransition for Slide {}
```

### TransitionType Enum

Each variant boxes the right trait bound, enforcing correctness:

```rust
pub enum TransitionType {
    Out(Box<dyn OutTransition>),
    In(Box<dyn InTransition>),
    OutIn {
        in_e: Box<dyn InTransition>,
        out_e: Box<dyn OutTransition>,
    },
    OneShot(Box<dyn OneShotTransition>),
}
```

`TransitionType::OneShot(Box::new(FadeIn))` will not compile because `FadeIn` does not
implement `OneShotTransition`. `OutIn` is composition -- mix and match any `OutTransition`
with any `InTransition`.

### TransitionRegistry

Maps `TypeId` of a transition effect type to the three systems that implement it.

```rust
#[derive(Resource)]
pub struct TransitionRegistry {
    entries: HashMap<TypeId, TransitionEntry>,
}

struct TransitionEntry {
    start: Box<dyn Fn(&mut World) + Send + Sync>,   // inserts StartingTransition<T>
    // The run and end systems are normal Bevy systems scheduled with run_if
    // on the corresponding marker resources. The registry only needs the
    // starter closure to bridge from type-erased TransitionType to concrete T.
}
```

The registry bridges the type-erased `TransitionType` (which holds `Box<dyn OutTransition>`)
to the concrete resource insertion. When a route fires with a transition:

1. Extract `TypeId` from the boxed trait object
2. Look up the `TransitionEntry` in the registry
3. Call the starter closure, which inserts `StartingTransition::<ConcreteEffect>`
4. The concrete effect's systems (scheduled with `run_if(resource_exists::<StartingTransition<T>>())`)
   take over

### Marker Resources

Generic over `T: Transition`:

```rust
#[derive(Resource)]
pub struct StartingTransition<T: Transition>(PhantomData<T>);

#[derive(Resource)]
pub struct RunningTransition<T: Transition>(PhantomData<T>);

#[derive(Resource)]
pub struct EndingTransition<T: Transition>(PhantomData<T>);
```

### Internal Messages

Transition systems communicate back to the crate via internal messages:

```rust
#[derive(Message, Clone)]
pub(crate) struct TransitionReady;   // start -> running

#[derive(Message, Clone)]
pub(crate) struct TransitionRunComplete;  // running -> ending

#[derive(Message, Clone)]
pub(crate) struct TransitionOver;    // ending -> done
```

These are `pub(crate)` -- the game never sends them. Transition effect systems send them
to signal phase completion.

### Active Transition Marker

```rust
#[derive(Resource)]
pub(crate) struct ActiveTransition;
```

Inserted when any transition begins, removed when it ends. Used by dispatch systems to
defer `ChangeState<S>` processing during active transitions.

### Orchestration Lifecycle

For `TransitionType::Out(effect)`:

```
Route fires with Transition::Out(FadeOut)
  1. Crate pauses Time<Virtual>
  2. Crate inserts ActiveTransition
  3. Crate sends TransitionStart<S> { from, to } to game
  4. Crate looks up TypeId::of::<FadeOut>() in TransitionRegistry
  5. Registry starter closure inserts StartingTransition::<FadeOut>
  6. fade_out_start system runs (spawns overlay, GlobalZIndex(i32::MAX - 1))
     -> sends TransitionReady
  7. Crate removes StartingTransition, inserts RunningTransition::<FadeOut>
  8. fade_out_run system runs each frame using Time<Real> (animates overlay)
     -> sends TransitionRunComplete when animation complete
  9. Crate removes RunningTransition, inserts EndingTransition::<FadeOut>
  10. fade_out_end system runs (cleanup if needed)
      -> sends TransitionOver
  11. Crate removes EndingTransition
  12. Crate applies NextState<S>::set_if_neq(destination) -- screen stays covered
  13. Crate sends StateChanged<S> { from, to }
  14. Crate sends TransitionEnd<S> { from, to }
  15. Crate removes ActiveTransition
      NOTE: For Out, virtual time is NOT unpaused here -- it stays paused
      until a matching In transition completes
```

For `TransitionType::In(effect)`:

```
Same steps 4-11 as Out, but:
  - Step 1: Virtual time is already paused (from the preceding Out)
  - After step 11: Crate unpauses Time<Virtual>
  - Steps 12-15: state change, messages, cleanup
```

For `TransitionType::OutIn { out_e, in_e }`:

```
  1. Crate pauses Time<Virtual>
  2-11. Run Out effect (out_e) through full lifecycle
  12. Apply state change (NextState<S>::set_if_neq)
  13-22. Run In effect (in_e) through full lifecycle
  23. Crate unpauses Time<Virtual>
  24. Send TransitionEnd<S>, remove ActiveTransition
```

For `TransitionType::OneShot(effect)`:

```
Same as Out lifecycle, but both old and new state content coexist during the effect.
State change happens before the effect starts (or concurrently -- effect-specific).
Virtual time pauses at start, unpauses at end.
```

### Time<Virtual> Pause/Unpause

- **Out**: pauses virtual time at start, does NOT unpause at end (screen stays covered)
- **In**: does NOT pause (already paused from Out), unpauses at end
- **OutIn**: pauses at start, unpauses at end (full cycle)
- **OneShot**: pauses at start, unpauses at end

The crate owns `Time<Virtual>` manipulation during transitions. Game pause is orthogonal:
if the user pauses during a transition, it's a no-op (virtual time already paused). Gate
the pause screen on `is_paused() AND NOT resource_exists::<ActiveTransition>()`.

### Deferred ChangeState During Transitions

While `ActiveTransition` exists, dispatch systems skip route execution. Any `ChangeState<S>`
messages that arrive during a transition are deferred -- they remain in the message buffer
and are processed on the next frame after the transition completes (when `ActiveTransition`
is removed and the dispatch system's `on_message` gate fires again).

This ensures animations play fully. Example: Loading completes behind the cover, route to
AnimateIn queues as `ChangeState`, transition reveals, then AnimateIn fires.

---

## 7. CleanupOnExit<S>

### Component

```rust
#[derive(Component)]
pub struct CleanupOnExit<S: States>(PhantomData<S>);

impl<S: States> CleanupOnExit<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S: States> Default for CleanupOnExit<S> {
    fn default() -> Self {
        Self::new()
    }
}
```

### Cleanup System

A generic system registered per state type that despawns all entities with
`CleanupOnExit<S>` when the state exits:

```rust
fn cleanup_on_exit<S: States>(
    mut commands: Commands,
    query: Query<Entity, With<CleanupOnExit<S>>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

### Registration

The plugin registers cleanup systems on `OnExit` for every variant of the state, or the
game can register for specific exit states. The simplest approach: register cleanup as a
system that runs on `StateTransition` whenever `State<S>` is removed or transitions.

Alternatively, the `register_state` method on the plugin can accept a "teardown variant"
parameter:

```rust
// Option A: cleanup fires on any exit of S (broadest)
plugin.register_state::<NodeState>()

// Option B: game explicitly wires cleanup to specific exit schedules
app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>);
app.add_systems(OnEnter(RunState::Teardown), cleanup_on_exit::<RunState>);
```

The spec uses `OnEnter(Teardown)` for cleanup -- the game wires these explicitly per state.
The crate provides the component and system; the game decides when cleanup runs.

---

## 8. Messages

All messages use `#[derive(Message)]` on generic structs. Confirmed working in Bevy 0.18
(see [research/generic-message-bevy.md](research/generic-message-bevy.md)).

### ChangeState<S>

```rust
#[derive(Message, Clone)]
pub struct ChangeState<S: States> {
    _phantom: PhantomData<S>,
}

impl<S: States> ChangeState<S> {
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

impl<S: States> Default for ChangeState<S> {
    fn default() -> Self {
        Self::new()
    }
}
```

Sent by the game to say "route me from my current state." Carries no destination -- the
routing table determines where to go. Each `ChangeState<NodeState>` and
`ChangeState<RunState>` are entirely separate message types with separate `Messages<T>`
resources.

### StateChanged<S>

```rust
#[derive(Message, Clone)]
pub struct StateChanged<S: States> {
    pub from: S,
    pub to: S,
}
```

Sent by the crate after every state change. Game systems can listen to coordinate audio,
analytics, loading indicators, etc.

### TransitionStart<S>

```rust
#[derive(Message, Clone)]
pub struct TransitionStart<S: States> {
    pub from: S,
    pub to: S,
}
```

Sent by the crate when a transition animation begins (virtual time is pausing). For OutIn:
sent once at the beginning, not at each phase.

### TransitionEnd<S>

```rust
#[derive(Message, Clone)]
pub struct TransitionEnd<S: States> {
    pub from: S,
    pub to: S,
}
```

Sent by the crate when a transition animation finishes (virtual time is resuming). For OutIn:
sent once at the end, not at each phase.

### Message Timing

| Route type | Messages sent |
|------------|---------------|
| No transition (plain ChangeState) | `StateChanged<S>` only |
| Out | `TransitionStart<S>` -> (animation) -> `StateChanged<S>` -> `TransitionEnd<S>` |
| In | `TransitionStart<S>` -> (animation) -> `StateChanged<S>` -> `TransitionEnd<S>` |
| OutIn | `TransitionStart<S>` -> Out anim -> `StateChanged<S>` -> In anim -> `TransitionEnd<S>` |
| OneShot | `TransitionStart<S>` -> (animation) -> `StateChanged<S>` -> `TransitionEnd<S>` |

### Registration

Each concrete instantiation must be registered separately:

```rust
app.add_message::<ChangeState<NodeState>>()
   .add_message::<ChangeState<RunState>>()
   .add_message::<ChangeState<GameState>>()
   .add_message::<StateChanged<NodeState>>()
   .add_message::<StateChanged<RunState>>()
   .add_message::<StateChanged<GameState>>()
   .add_message::<TransitionStart<NodeState>>()
   .add_message::<TransitionStart<RunState>>()
   // etc. -- one registration per state type per message type
```

The plugin's `register_state::<S>()` method handles all four message registrations for
a given `S` automatically.

---

## 9. Plugin

### RantzLifecyclePlugin

Builder-pattern plugin. Per the `rantzsoft_*` naming convention: `RantzLifecyclePlugin`.

```rust
pub struct RantzLifecyclePlugin {
    state_registrations: Vec<Box<dyn FnOnce(&mut App) + Send + Sync>>,
    custom_transitions: Vec<Box<dyn FnOnce(&mut App) + Send + Sync>>,
}

impl RantzLifecyclePlugin {
    pub fn new() -> Self {
        Self {
            state_registrations: Vec::new(),
            custom_transitions: Vec::new(),
        }
    }

    /// Register all lifecycle infrastructure for state type S:
    /// - RoutingTable<S> resource
    /// - Message types (ChangeState<S>, StateChanged<S>, TransitionStart<S>, TransitionEnd<S>)
    /// - Dispatch systems (message-triggered + condition-triggered)
    /// - CleanupOnExit<S> component is always available (no per-state registration needed)
    pub fn register_state<S: States + Eq + Hash + Clone>(mut self) -> Self {
        self.state_registrations.push(Box::new(|app| {
            app.init_resource::<RoutingTable<S>>()
               .add_message::<ChangeState<S>>()
               .add_message::<StateChanged<S>>()
               .add_message::<TransitionStart<S>>()
               .add_message::<TransitionEnd<S>>()
               .add_systems(Update, (
                   dispatch_message_routes::<S>
                       .run_if(on_message::<ChangeState<S>>()),
                   dispatch_condition_routes::<S>,
               ));
        }));
        self
    }

    /// Register a custom transition effect type with its three implementation systems.
    pub fn register_custom_transition<T: Transition + 'static>(
        mut self,
        start: impl IntoSystemConfigs<()> + 'static,
        run: impl IntoSystemConfigs<()> + 'static,
        end: impl IntoSystemConfigs<()> + 'static,
    ) -> Self {
        self.custom_transitions.push(Box::new(|app| {
            // Add systems gated on marker resources
            app.add_systems(Update, (
                start.run_if(resource_exists::<StartingTransition<T>>),
                run.run_if(resource_exists::<RunningTransition<T>>),
                end.run_if(resource_exists::<EndingTransition<T>>),
            ));
            // Register in TransitionRegistry
            app.world_mut()
               .resource_mut::<TransitionRegistry>()
               .register::<T>();
        }));
        self
    }
}

impl Plugin for RantzLifecyclePlugin {
    fn build(&self, app: &mut App) {
        // Initialize shared resources
        app.init_resource::<TransitionRegistry>();

        // Register built-in transitions (FadeIn, FadeOut, Slide, etc.)
        register_builtin_transitions(app);

        // Register internal messages
        app.add_message::<TransitionReady>()
           .add_message::<TransitionRunComplete>()
           .add_message::<TransitionOver>();

        // Add transition orchestration system
        app.add_systems(Update, orchestrate_transitions);

        // Apply per-state registrations
        for registration in self.state_registrations.drain(..) {
            registration(app);
        }

        // Apply custom transition registrations
        for registration in self.custom_transitions.drain(..) {
            registration(app);
        }
    }
}
```

### Usage in Game

```rust
app.add_plugins(
    RantzLifecyclePlugin::new()
        .register_state::<AppState>()
        .register_state::<GameState>()
        .register_state::<MenuState>()
        .register_state::<RunState>()
        .register_state::<NodeState>()
        .register_state::<ChipSelectState>()
        .register_state::<RunEndState>()
        .register_custom_transition::<MyCustomWipe>(
            custom_wipe_start,
            custom_wipe_run,
            custom_wipe_end,
        )
);

// Then add routes:
app.add_route(Route::from(NodeState::Loading).to(NodeState::AnimateIn))
   .add_route(Route::from(NodeState::AnimateIn).to(NodeState::Playing))
   // ...
```

### Built-In vs. Custom Transitions

Built-in transitions (FadeIn, FadeOut, Slide, Dissolve, etc.) are registered automatically
by the plugin's `build()` method. The game does not need to register them.

Custom transitions use `.register_custom_transition::<T>()` on the plugin builder:

- **Traits**: `pub` -- game can implement `Transition`, `InTransition`, `OutTransition`,
  `OneShotTransition` for custom effects
- **`TransitionRegistry::register`**: `pub(crate)` -- internal plumbing, not game-facing
- **`.register_custom_transition::<T>()`**: `pub` -- game's entry point for custom effects

---

## 10. Startup Validation

After all plugins have built, the crate validates that every route referencing a transition
type has that type registered in the `TransitionRegistry`.

```rust
fn validate_routes(world: &World) {
    let registry = world.resource::<TransitionRegistry>();

    // For each registered RoutingTable<S>, scan all routes
    // Check that every TransitionType's inner TypeId exists in the registry
    // Panic with a clear message if any are missing:
    //   "Route from {:?} references transition {:?} which is not registered
    //    in TransitionRegistry. Register it via .register_custom_transition::<T>()"
}
```

This runs as a startup system (or in `Plugin::finish`) so missing registrations are caught
immediately, not at the first transition trigger.

### What Is Validated

- Every `TransitionType::Out(t)` -- `TypeId::of_val(&*t)` must exist in registry
- Every `TransitionType::In(t)` -- same
- Every `TransitionType::OutIn { in_e, out_e }` -- both must exist
- Every `TransitionType::OneShot(t)` -- same

Dynamic transitions (`with_dynamic_transition`) cannot be validated at startup because the
`TransitionType` is determined at runtime. These are validated at dispatch time instead --
panic if the returned transition type's `TypeId` is not in the registry.

---

## Appendix: Key Research References

| Topic | File |
|-------|------|
| Routing approaches (7 evaluated, one-shot + resource_scope chosen) | [research/declarative-routing.md](research/declarative-routing.md) |
| Avoiding exclusive world access, on_message gating, multiple messages | [research/routing-without-exclusive-world.md](research/routing-without-exclusive-world.md) |
| Generic #[derive(Message)] confirmed for Bevy 0.18 | [research/generic-message-bevy.md](research/generic-message-bevy.md) |
| when() condition approaches (polling vs event-driven) | [research/event-driven-route-conditions.md](research/event-driven-route-conditions.md) |
| Screen transition overlay patterns, GlobalZIndex | [research/bevy-transition-patterns.md](research/bevy-transition-patterns.md) |
| SubStates 4-level nesting confirmed, no depth limit | [research/substates-nesting-depth.md](research/substates-nesting-depth.md) |
| Time<Virtual>::pause() as primary pause mechanism | [research/bevy-pause-patterns.md](research/bevy-pause-patterns.md) |
| ScreenLifecycle trait -- associated methods, derive macro deferred | [research/enum-trait-constraints.md](research/enum-trait-constraints.md) |
