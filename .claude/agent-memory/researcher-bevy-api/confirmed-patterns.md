# Confirmed Bevy 0.18.1 API Patterns

Verified against docs.rs/bevy/0.18.1 and official source.

---

## UI Z-Ordering — GlobalZIndex and ZIndex

### `GlobalZIndex(i32)` — cross-hierarchy overlay ordering

```rust
use bevy::prelude::GlobalZIndex;  // re-exported in prelude

// Render above ALL other UI nodes globally (same pattern as Bevy FPS overlay):
GlobalZIndex(i32::MAX - 1)
// FPS overlay uses i32::MAX - 32 "so you can render on top of it if you really need to"
```

- `GlobalZIndex` allows a Node to escape the implicit draw ordering of the UI layout tree
- Positive values render ON TOP of nodes without GlobalZIndex or lower values
- Negative values render BELOW nodes without GlobalZIndex or higher values
- For siblings with same GlobalZIndex: the one with greater local `ZIndex` wins
- `ZIndex` alone only affects ordering among siblings — use `GlobalZIndex` for cross-hierarchy overlays
- Verified from `docs.rs/bevy/0.18.0/bevy/prelude/struct.GlobalZIndex.html`
- Confirmed pattern from `docs.rs/bevy_dev_tools/0.18.1/src/bevy_dev_tools/fps_overlay.rs`

### Full-screen overlay spawn pattern (confirmed working)

```rust
commands.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    },
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
    GlobalZIndex(i32::MAX - 1),  // covers all other UI including HUD
));
```

---

## StateTransitionEvent (Bevy 0.18)

```rust
// Implements Message — read with MessageReader<StateTransitionEvent<S>>
pub struct StateTransitionEvent<S: States> {
    pub exited: Option<S>,
    pub entered: Option<S>,
    pub allow_same_state_transitions: bool,
}
```

Fires AFTER the transition completes (after OnEnter/OnExit schedules run).

### BREAKING CHANGE in Bevy 0.18

`next_state.set(S)` now ALWAYS fires `OnEnter`/`OnExit`, even when setting the same state value.
Use `next_state.set_if_neq(S)` for the old behavior (only transition if the state is different).

---

## Time API

### `Time<Virtual>` — controlling game speed

```rust
// Access:
fn my_system(mut virtual_time: ResMut<Time<Virtual>>) { ... }

// Set speed (panics if negative or non-finite):
virtual_time.set_relative_speed(0.3_f32);   // 30% speed slow-motion
virtual_time.set_relative_speed_f64(0.3_f64);

// Read current speed:
let speed: f32 = virtual_time.relative_speed();
let effective: f32 = virtual_time.effective_speed(); // 0.0 when paused

// Pause / unpause:
virtual_time.pause();
virtual_time.unpause();
let paused: bool = virtual_time.is_paused();
```

### `Time<Virtual>` affects `Time<Fixed>` / FixedUpdate

Confirmed from source: `run_fixed_main_schedule` reads `Time<Virtual>.delta()` and
accumulates it into `Time<Fixed>`. Setting `set_relative_speed(0.3)` means FixedUpdate
runs at ~30% of normal frequency. Each fixed step still has the same `timestep()` duration
(default 64 Hz ≈ 15.6ms) — you get fewer steps, not slower steps.

### `Time<Real>` — always wall-clock, unaffected by speed/pause

```rust
fn my_system(real_time: Res<Time<Real>>) {
    let wall_delta: f32 = real_time.delta_secs();
    let wall_elapsed: f32 = real_time.elapsed_secs();
}
```

Always use `Time<Real>` for: ramp timers, UI animations, audio timing, anything that
must not be affected by game speed changes.

### Plain `Res<Time>` — context-dependent alias

- In `Update`: behaves like `Time<Virtual>`
- In `FixedUpdate`: behaves like `Time<Fixed>`

### Smooth ramp-in/ramp-out for time dilation

No built-in ramp. Implement with a system in `Update` that reads `Time<Real>` (not `Time<Virtual>`!)
and calls `set_relative_speed()` each frame. Using `Time<Virtual>` for the ramp timer creates a
recursive slow-down bug where the ramp itself slows as speed decreases.

```rust
// Use Time<Real> for the ramp timer:
fn ramp_system(real: Res<Time<Real>>, mut virt: ResMut<Time<Virtual>>, mut state: ResMut<Ramp>) {
    state.elapsed += real.delta_secs();  // real time, not virtual!
    let t = (state.elapsed / state.duration).clamp(0.0, 1.0);
    let t_smooth = t * t * (3.0 - 2.0 * t); // smooth-step
    virt.set_relative_speed(state.start + (state.target - state.start) * t_smooth);
}
```

---

## Message System (Bevy 0.18)

Verified against docs.rs/bevy_ecs/0.18.1, docs.rs/bevy_ecs_macros/0.18.1, github.com/bevyengine/bevy/tree/v0.18.1.

### `#[derive(Message)]` on generic structs — CONFIRMED WORKS

The `derive_message` macro in `bevy_ecs_macros/src/message.rs`:
1. Calls `ast.generics.split_for_impl()` — preserves all user generics
2. Appends `Self: Send + Sync + 'static` to the where clause
3. Emits an empty impl block: `impl<T> Message for Foo<T> where Self: Send + Sync + 'static {}`

**Proof**: `StateTransitionEvent<S: States>` in bevy_state uses `#[derive(..., Message)]` on a
generic struct — verified from bevy_state source.

**Manual impl also works**: `AssetEvent<A>` uses `impl<A> Message for AssetEvent<A> where A: Asset`.

`Message` trait bounds: `Send + Sync + 'static` only.
`MessageReader<E>`: bound `E: Message` only — no extra bounds.
`MessageWriter<E>`: bound `E: Message` only — no extra bounds.

`Messages<ChangeState<NodeState>>` and `Messages<ChangeState<RunState>>` are distinct resources
(different `TypeId`). Each instantiation needs its own `app.add_message::<T>()` call.

### Core types

| Type | Role |
|------|------|
| `Messages<T>` | `Resource` — the actual message storage (double-buffered) |
| `MessageWriter<T>` | `SystemParam` — thin wrapper around `ResMut<Messages<T>>` |
| `MessageReader<T>` | `SystemParam` — reads from `Messages<T>` with cursor tracking |
| `MessageMutator<T>` | `SystemParam` — mutable read with cursor tracking |
| `MessageCursor<T>` | Tracks per-reader position in the message buffer |

`MessageWriter<T>` has no extra logic: its `write()` method just calls `self.messages.write(message)`.

### Registering a message type

```rust
app.add_message::<MyMessage>();
// Must be called before any system reads/writes the message.
// Inserts Messages<MyMessage> as a resource and schedules update system.
```

### Writing messages from a system

```rust
fn my_system(mut writer: MessageWriter<MyMessage>) {
    writer.write(MyMessage { ... });
}
```

### Writing messages directly from &mut World (in tests)

`Messages<T>` implements `Resource` directly, so you can write to it from any `&mut World`:

```rust
// Option 1 — resource_mut (most explicit, recommended for tests):
app.world_mut()
    .resource_mut::<Messages<MyMessage>>()
    .write(MyMessage { ... });

// Option 2 — World::write_message convenience method:
app.world_mut().write_message(MyMessage { ... });

// Batch variant:
app.world_mut()
    .resource_mut::<Messages<MyMessage>>()
    .write_batch([msg1, msg2]);
```

`World::write_message` / `write_message_batch` / `write_message_default` are confirmed
in the World method list on docs.rs. The underlying implementation (in DeferredWorld) calls
`get_resource_mut::<Messages<E>>()` and delegates to `write_batch` — it logs an error and
returns `None` if the type was not registered.

### Reading messages from a system

```rust
fn my_system(mut reader: MessageReader<MyMessage>) {
    for msg in reader.read() {
        // msg: &MyMessage
    }
}
```

### The `#[cfg(test)]` workaround used in this project (before knowing the above)

The project currently uses a Resource + helper-system pattern to inject test messages:

```rust
// Old workaround in tests:
#[derive(Resource)]
struct TestMessage(Option<DamageCell>);

fn enqueue_from_resource(res: Res<TestMessage>, mut writer: MessageWriter<DamageCell>) {
    if let Some(msg) = res.0.clone() { writer.write(msg); }
}

app.insert_resource(TestMessage(Some(msg)));
app.add_systems(FixedUpdate, enqueue_from_resource.before(handle_cell_hit));
tick(&mut app);
```

This can be replaced with the direct resource_mut approach — no helper system or TestMessage resource needed.

### Message update / buffer swap

`Messages::update()` swaps double buffers once per frame. This is handled automatically by
`message_update_system` (scheduled in `app.add_message()`). Tests using `app.update()` or
`tick()` will have messages visible to readers on the same update tick they are written.

### Module path

```rust
use bevy::ecs::message::{Messages, MessageWriter, MessageReader, MessageCursor};
// All re-exported in bevy::prelude
use bevy::prelude::*;
```

---

## Queries — QueryData Derive (custom named query structs)

Verified against `docs.rs/bevy/0.18.0` and `github.com/bevyengine/bevy/blob/v0.18.0/examples/ecs/custom_query_param.rs`.

### Read-only struct (no mutable components)

```rust
use bevy::ecs::query::QueryData;

#[derive(QueryData)]
struct MyQuery {
    entity: Entity,
    position: &'static Transform,
    velocity: Option<&'static Velocity>,
}
```

- No attribute needed for read-only structs.
- Iterating with `&query` or `&mut query` both yield the same read-only item type.

### Mutable struct (at least one `&'static mut` field)

```rust
#[derive(QueryData)]
#[query_data(mutable)]
struct MyQuery {
    entity: Entity,
    position: &'static mut Transform,
    velocity: Option<&'static mut Velocity>,
    speed: &'static Speed,  // read-only field is fine in a mutable struct
}
```

**The `#[query_data(mutable)]` attribute is required whenever any field is `&'static mut`.**

### Generated types (for a struct named `MyQuery`)

| Type | When generated | Use |
|------|---------------|-----|
| `MyQueryItem<'w, 's>` | Always | The item when iterating `&mut query` |
| `MyQueryReadOnly` | Only with `mutable` | Read-only variant usable in `Query<MyQueryReadOnly>` |
| `MyQueryReadOnlyItem<'w, 's>` | Only with `mutable` | The item when iterating `&query` (immutable borrow of a mutable query) |

### Iterating in a system

```rust
// Immutable iteration — yields read-only items
fn my_system(query: Query<MyQuery>) {
    for item in &query {
        // item fields are all &T (read-only refs)
        let _ = item.entity;
        let _ = item.position;
    }
}

// Mutable iteration — yields mutable items
fn my_mut_system(mut query: Query<MyQuery>) {
    for mut item in &mut query {
        // mutable fields are Mut<T>
        item.position.translation.x += 1.0;
    }
}

// Explicit item type annotation (useful for clarity)
fn my_explicit_system(mut query: Query<MyQuery>) {
    for e in &mut query {
        let e: MyQueryItem<'_, '_, > = e;
        // ...
    }
}
```

### Deriving Debug on the generated item types

Pass extra derives via the attribute:

```rust
#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
struct MyQuery { ... }
```

### Nesting QueryData structs

Fields can themselves be another `#[derive(QueryData)]` struct — no special syntax needed.

**Nesting a mutable QueryData inside another mutable QueryData:**

```rust
#[derive(QueryData)]
#[query_data(mutable)]
struct Inner {
    a: &'static mut ComponentA,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct Outer {
    b: &'static mut ComponentB,
    inner: Inner,  // nested mutable QueryData — no annotation needed
}
```

**How mutable nesting resolves** (verified from macro source `query_data.rs` at v0.18.0):

- Mutable item struct (`OuterItem`): macro uses field types directly. `inner` field resolves to `InnerItem` — which **has `Mut<T>` fields**. Mutable access cascades automatically.
- Read-only item struct (`OuterReadOnlyItem`): macro applies `<Inner as QueryData>::ReadOnly`, resolving to `InnerReadOnly` → `InnerReadOnlyItem`. All fields become `&T`.

No special syntax on the nested field. The outer struct's mutable/read-only context automatically determines which generated type the nested field resolves to.

**Nesting a read-only struct inside a mutable one** (also valid):

```rust
#[derive(QueryData)]
struct Inner { c: &'static ComponentC }  // no mutable attribute

#[derive(QueryData)]
#[query_data(mutable)]
struct Outer {
    a: &'static mut ComponentA,
    inner: Inner,  // nested read-only QueryData — always read-only regardless of outer context
}
```

### Generics

Supported — type parameters must be `Component`:

```rust
#[derive(QueryData)]
struct GenericQuery<T: Component, P: Component> {
    t: &'static T,
    p: &'static P,
}
```

### `Option<>` fields

Supported for optional components, exactly like tuple queries:

```rust
active_boost: Option<&'static ActiveSpeedBoosts>,
```

### Import path

```rust
use bevy::ecs::query::QueryData;
// or via prelude if re-exported (verify — safe to use full path)
```

### `With<T>` CANNOT be a QueryData field — filter only

`With<T>` implements `QueryFilter` (and `WorldQuery`) but does NOT implement `QueryData`.
Using `With<T>` as a field in a `#[derive(QueryData)]` struct is a compile error.

```rust
// WRONG — does not compile:
#[derive(QueryData)]
struct SpatialData {
    position: &'static mut Position2D,
    _marker: With<Spatial>,  // ERROR: With<T> does not implement QueryData
}
```

Filters must always be applied at the `Query<>` level as the second generic parameter:

```rust
// CORRECT — filter at the Query level:
fn my_system(query: Query<SpatialData, With<Spatial>>) { ... }
```

### `Has<T>` — presence check that IS valid inside QueryData

`Has<T>` implements `QueryData` (not `QueryFilter`). It resolves to `bool` — true if the
entity has component `T`, false otherwise. Does not conflict with mutable access unlike `Option<&T>`.

```rust
#[derive(QueryData)]
#[query_data(mutable)]
struct MyQuery {
    position: &'static mut Transform,
    has_boost: Has<SpeedBoost>,  // resolves to bool — valid QueryData field
}

fn my_system(mut query: Query<MyQuery>) {
    for mut item in &mut query {
        if item.has_boost {
            // ...
        }
    }
}
```

Use `Has<T>` when you want to check component presence in the data struct itself.
Use `With<T>` / `Without<T>` as a filter when you only want to match entities that have/lack the component.

### Key rules summary

1. Any `&'static mut` field → **requires** `#[query_data(mutable)]` on the struct
2. Read-only fields (`&'static T`) are valid in a mutable struct
3. Field type is always `&'static T` or `&'static mut T` (with the `'static` lifetime) in the struct definition
4. The actual item type in system iteration uses short lifetimes (`'w`) — Bevy handles this
5. Tuple fields work: `Option<(&'static ComponentB, &'static ComponentZ)>`
6. `With<T>` is a filter ONLY — not valid as a QueryData field. Use `Query<D, With<T>>` instead.
7. `Has<T>` IS valid as a QueryData field — resolves to `bool` at query time.

---

## State — States, SubStates, ComputedStates (Bevy 0.18.1)

Verified against docs.rs/bevy/0.18.1, bevy v0.18.0 source, official examples.

### `States` trait (top-level, independent)

```rust
pub trait States: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {
    const DEPENDENCY_DEPTH: usize = 1;
}
```

Multiple independent `States` can coexist in one app — "orthogonal dimensions."

```rust
app.init_state::<GameState>()
   .init_state::<PauseState>(); // completely independent
```

Both need `Default` (sets initial state), plus `Clone, Copy, PartialEq, Eq, Hash, Debug`.
Registered via `AppExtStates::init_state::<S>()` or `insert_state(S)`.

### `SubStates` trait (hierarchical, requires source)

```rust
pub trait SubStates: States {
    type SourceStates: StateSet;
    fn should_exist(sources: Self::SourceStates) -> Option<Self>;
}
```

Derive macro + `#[source(ParentState = ParentState::Variant)]` sets up the source.
Only exists when the source state condition is met; resource removed when condition fails.
`SourceStates` can be a single type or a tuple of multiple types.
SubStates CANNOT be independent (source-free) — that requires `States`.

Registration: `app.add_sub_state::<S>()` after the parent state is initialized.

### `ComputedStates` trait (derived, no manual transitions)

```rust
pub trait ComputedStates: 'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug {
    type SourceStates;
    fn compute(sources: Self::SourceStates) -> Option<Self>;
    const ALLOW_SAME_STATE_TRANSITIONS: bool = true;
}
```

Does NOT require `Default`. Automatically recomputed when any source changes.
Returns `None` to remove the state resource from the world (state inactive).
`SourceStates` can be a single type, `Option<T>`, or tuple of multiple types.

Registration: `app.add_computed_state::<S>()`.

### State reading and transitioning

```rust
fn my_system(
    current: Res<State<GameState>>,          // read current state
    mut next: ResMut<NextState<GameState>>,  // queue transition
) {
    let s: &GameState = current.get();
    next.set(GameState::Playing);
}
```

### `in_state` condition

```rust
pub fn in_state<S: States>(state: S) -> impl FnMut(Option<Res<'_, State<S>>>) + Clone;
```

Works in any schedule — `Update`, `FixedUpdate`, etc.

```rust
.add_systems(FixedUpdate, physics.run_if(in_state(NodeState::Playing)))
.add_systems(Update, ui_system.run_if(in_state(GameState::Run)))
```

### configure_sets is per-schedule

**CRITICAL**: A `run_if` on a SystemSet in one schedule does NOT propagate to other
schedules. Must configure separately:

```rust
// MUST call configure_sets on EACH schedule independently:
app.configure_sets(Update, GameplaySystems.run_if(not_paused));
app.configure_sets(FixedUpdate, GameplaySystems.run_if(not_paused));
```

`in_state` accepts `Option<Res<State<S>>>` as its param so it returns `false` (not panic)
if the state resource doesn't exist (e.g., SubStates not yet active).

### AppExtStates signatures

```rust
fn init_state<S: States + Default>(&mut self) -> &mut Self;
fn insert_state<S: States>(&mut self, state: S) -> &mut Self;
fn add_sub_state<S: SubStates>(&mut self) -> &mut Self;
fn add_computed_state<S: ComputedStates>(&mut self) -> &mut Self;
```

### SubStates nesting depth — verified 0.18.1

**SubStates can source from other SubStates** — no depth limit. The chain works because
`SubStates: States`, so any SubStates type satisfies `InnerStateSet for S: States`, which
satisfies `StateSet`. The bound is `S: States`, not `S: States + !SubStates`.

DEPENDENCY_DEPTH auto-increments: each level adds 1 via
`SourceStates::SET_DEPENDENCY_DEPTH + 1`. The derive macro generates this automatically.

**Teardown order** (innermost exits first):
```
NodeState::OnExit → RunState::OnExit → GameState::OnExit → AppState::OnExit
```

**Enter order** (outermost enters first):
```
AppState::OnEnter → GameState::OnEnter → RunState::OnEnter → NodeState::OnEnter
```

**Valid 4-level hierarchy** (direct chaining, no ComputedState intermediary needed for
simple variant matching):
```rust
app.init_state::<AppState>()
   .add_sub_state::<GameState>()   // source: AppState = AppState::Game
   .add_sub_state::<RunState>()    // source: GameState = GameState::Run
   .add_sub_state::<NodeState>();  // source: RunState = RunState::Node
```

ComputedState intermediary is only needed when source uses struct variants with fields.
Official examples only show 1-level nesting, but the type constraints confirm 4-level works.

`try_run_schedule` is used internally — missing OnEnter/OnExit handlers do not panic.

---

## on_message run condition (Bevy 0.18.1)

Verified from source: `github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_ecs/src/schedule/condition.rs`

```rust
// Exact signature:
pub fn on_message<M: Message>(reader: MessageReader<'_, '_, M>) -> bool

// Module path:
use bevy::ecs::schedule::common_conditions::on_message;
// Also re-exported in bevy::prelude::*

// Usage:
.add_systems(Update, my_system.run_if(on_message::<MyMessage>()))
```

**CRITICAL**: `on_message` uses its own `MessageReader` with its own `Local<MessageCursor<M>>`.
Each `MessageReader` instance (whether in a run condition or system body) has an **independent cursor**.
The condition advancing its cursor does NOT consume messages from the system body's reader.

Source implementation:
```rust
pub fn on_message<M: Message>(mut reader: MessageReader<M>) -> bool {
    reader.read().count() > 0
    // "The messages need to be consumed, so that there are no false positives
    // on subsequent calls of the run condition. Simply checking is_empty would not be enough."
}
```

The condition returns `true` when new messages exist AND advances the condition's own cursor to prevent
re-firing on the same message batch. The system body's `MessageReader` still sees all messages.

## SystemParam derive

TODO: add from next research session.

---

## Component spawning

TODO: add from next research session.

---

## World access — One-shot systems and Commands::run_system (Bevy 0.18.1)

Verified against docs.rs/bevy/0.18.1, github.com/bevyengine/bevy/tree/v0.18.1.

### One-shot system registration and execution

```rust
// Register at app build time — returns SystemId (Copy + Send + Sync)
let id: SystemId = app.world_mut().register_system(my_fn);

// Or from &mut World directly
let id: SystemId = world.register_system(my_fn);

// Run from Commands in a normal system (deferred — executes at ApplyDeferred)
fn my_system(mut commands: Commands, ids: Res<RouteSystemIds>) {
    commands.run_system(ids.some_route);       // no input
    commands.run_system_with(ids.other, val);  // with input
}

// Run immediately from &mut World (exclusive context only)
world.run_system(id)
world.run_system_with(id, input)
```

- `SystemId<I = (), O = ()>` is `Copy + Send + Sync` — safe to store in Resources
- `Commands::run_system` is deferred (runs at next ApplyDeferred sync point, same frame)
- No return value from `Commands::run_system` — one-shot system writes to resources/components directly
- One-shot systems can read any `SystemParam` (Res, ResMut, Query, etc.) — no need for &mut World

### StateTransition schedule placement

`StateTransition` is inserted **after PreUpdate** in the main schedule:
- Frame order: `PreUpdate` → `StateTransition` → `FixedUpdate` → `Update` → `PostUpdate`
- A `NextState` queued during `FixedUpdate` or `Update` takes effect in the NEXT frame's `StateTransition`
- A `NextState` queued during `PreUpdate` takes effect in the SAME frame's `StateTransition`

### NextState has one slot — last set() wins

`NextState<S>` stores a single pending value. Multiple `set_if_neq()` calls in the same frame: only the last one takes effect. Process at most one routing message per frame and warn on duplicates.

---

## Let-chains

TODO: add from next research session.

---

## Bundle trait — introspection and iteration (Bevy 0.18.1)

Verified against docs.rs/bevy/0.18.1/bevy/ecs/bundle/.

### Trait definition

```rust
pub unsafe trait Bundle: DynamicBundle + Send + Sync + 'static {
    // Required method — returns None for each component not yet registered
    fn get_component_ids(
        components: &Components,
    ) -> impl Iterator<Item = Option<ComponentId>>;
}
```

- `unsafe trait` — manual impls are unsupported; always use `#[derive(Bundle)]`
- NOT dyn-compatible (cannot use as trait object)
- Supertrait `DynamicBundle` is where `get_components` (low-level pointer-based extraction) lives

### DynamicBundle supertrait

```rust
pub trait DynamicBundle: Sized {
    type Effect;

    // Low-level: moves component pointers out, calling func per component in
    // the same order as get_component_ids — requires unsafe, raw pointer work
    unsafe fn get_components(
        ptr: MovingPtr<'_, Self>,
        func: &mut impl FnMut(StorageType, OwningPtr<'_>),
    );

    // Runs post-insertion effects on the entity
    unsafe fn apply_effect(
        ptr: MovingPtr<'_, MaybeUninit<Self>>,
        entity: &mut EntityWorldMut<'_>,
    );
}
```

### Tuple impls

- `()` implements Bundle (empty set)
- Tuples of up to 16 items where each item: Bundle → impl Bundle
- Nest tuples for >15 components: `((A, B, C, ..., O), P, Q)`

### BundleInfo — inspection after World registration

Obtained only via `World::bundles()` — cannot be constructed directly.

```rust
// world.bundles() returns &Bundles
let bundles: &Bundles = world.bundles();
let bundle_id: Option<BundleId> = bundles.get_id(TypeId::of::<MyBundle>());
let info: Option<&BundleInfo> = bundle_id.and_then(|id| bundles.get(id));

// BundleInfo methods:
info.id() -> BundleId
info.explicit_components() -> &[ComponentId]   // defined in the bundle struct
info.required_components() -> &[ComponentId]   // pulled in by #[require(...)]
info.contributed_components() -> &[ComponentId] // explicit + required combined
info.iter_explicit_components() -> impl Iterator<Item = ComponentId>
info.iter_contributed_components() -> impl Iterator<Item = ComponentId>
info.iter_required_components() -> impl Iterator<Item = ComponentId>
```

BundleInfo is only populated after the bundle type has been registered (i.e., spawned or
explicitly registered). Before that, `bundles.get_id(TypeId::of::<MyBundle>())` returns `None`.

### get_component_ids with a standalone Components

`Components` implements `Default`, so you CAN construct one without a World:

```rust
let mut components = Components::default();
// But: component IDs are only registered when you call components.register_component::<T>()
// A fresh Components is empty, so get_component_ids yields None for everything
```

For practical use, `Components` must be populated (via `World`) before IDs exist.

### Answers to the five common questions

1. **Can you destructure `impl Bundle`?**
   No. `impl Bundle` is an opaque return type — you cannot pattern-match or destructure it.
   A concrete bundle struct can be destructured normally if its fields are public.

2. **What methods does Bundle provide?**
   Only `get_component_ids(components: &Components) -> impl Iterator<Item = Option<ComponentId>>`.
   The `DynamicBundle` supertrait provides `get_components` and `apply_effect`, but those
   are low-level unsafe pointer operations intended for ECS internals, not user code.

3. **Can you iterate over components in a Bundle?**
   Not safely outside the ECS. `DynamicBundle::get_components` consumes the bundle via
   `MovingPtr` (moves component data out via raw pointers) — it is unsafe internals.
   The only user-facing iteration is `BundleInfo::iter_explicit_components()`, which gives
   `ComponentId`s only (not the actual component values), and requires a World.

4. **Inspect bundle contents without spawning into a World?**
   Limited. `Bundle::get_component_ids` accepts a `&Components` and yields
   `Option<ComponentId>` per component. You can construct a `Components::default()` and
   register components into it manually, then call this — but it is awkward and not the
   intended usage. There is NO way to get the actual component VALUES out of a bundle
   without spawning into a World.

5. **How to test bundle contains right components without World?**
   The idiomatic approach IS to use a World. Create a minimal `World::default()`, spawn
   the bundle, then query for expected components. This is the standard Bevy test pattern.
   There is no reflection-based "inspect without spawning" API.

### Testing bundles — the correct Bevy pattern

```rust
#[test]
fn my_bundle_has_expected_components() {
    let mut world = World::default();
    let entity = world.spawn(MyBundle { ... }).id();

    // Assert presence of expected components
    assert!(world.get::<ComponentA>(entity).is_some());
    assert!(world.get::<ComponentB>(entity).is_some());
    assert_eq!(*world.get::<ComponentA>(entity).unwrap(), expected_value);
}
```

---

## Run Condition Combinators (Bevy 0.18.1)

Verified from `github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_ecs/src/schedule/condition.rs`.

All run conditions implement `SystemCondition` (an alias for `IntoSystem<In, bool, Marker>` where
the system is `ReadOnlySystem`). The trait provides combinator methods:

```rust
fn and<M, C: SystemCondition<M, In>>(self, and: C) -> And<Self::System, C::System>
fn or<M, C: SystemCondition<M, In>>(self, or: C) -> Or<Self::System, C::System>
fn nand<M, C: SystemCondition<M, In>>(self, nand: C) -> Nand<Self::System, C::System>
fn nor<M, C: SystemCondition<M, In>>(self, nor: C) -> Nor<Self::System, C::System>
```

`.and()` short-circuits: if left is false, right is never evaluated.
`.or()` short-circuits: if left is true, right is never evaluated.

```rust
// Example: fire only when message arrived AND in a specific state
.run_if(on_message::<NodeExited>().and(in_state(RunState::Node)))
```

---

## Resource Change Detection Run Conditions (Bevy 0.18.1)

Verified from source and docs.rs/bevy/0.18.1.

### `resource_changed<T>` — panics if resource absent

```rust
pub fn resource_changed<T>(res: Res<'_, T>) -> bool where T: Resource
// Implementation: res.is_changed()
// PANICS if T does not exist in the world
```

### `resource_exists_and_changed<T>` — safe variant

```rust
pub fn resource_exists_and_changed<T>(res: Option<Res<'_, T>>) -> bool where T: Resource
// Implementation: match res { Some(r) => r.is_changed(), None => false }
// Returns false (no panic) if T does not exist
```

"Changed" means "mutably dereferenced since the condition last ran" — Bevy does not compare
values. `ResMut<T>` access sets changed even if no mutation occurred.

### `resource_changed_or_removed<T>` — detects removal too

```rust
pub fn resource_changed_or_removed<T>(
    res: Option<Res<'_, T>>,
    existed: Local<'_, bool>,
) -> bool where T: Resource
// Returns false if resource does not exist (uses Option)
```

### `Res<T>` implements `DetectChanges`

`Res<T>` directly exposes: `is_changed() -> bool`, `is_added() -> bool`, `last_changed() -> Tick`,
`added() -> Tick`. `Ref<T>` is for COMPONENTS only — do NOT use `Ref` for resources.

---

## State-Related Run Conditions (Bevy 0.18.1)

Verified from `github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_state/src/condition.rs`.

```rust
pub fn state_exists<S: States>(current_state: Option<Res<State<S>>>) -> bool {
    current_state.is_some()
}

pub fn in_state<S: States>(state: S) -> impl FnMut(Option<Res<State<S>>>) -> bool + Clone {
    move |current_state| match current_state {
        Some(s) => *s == state,
        None => false,
    }
}

pub fn state_changed<S: States>(current_state: Option<Res<State<S>>>) -> bool {
    current_state.map_or(false, |s| s.is_changed())
}
```

**Critical gotcha**: `state_changed<S>` returns `false` when `State<S>` is REMOVED — the Option
is None so it returns false. It does NOT detect SubState removal (teardown). Use
`condition_changed_to(false, state_exists::<S>())` to detect removal.

---

## `condition_changed` / `condition_changed_to` (Bevy 0.18.1)

Verified from source.

```rust
pub fn condition_changed<Marker, CIn, C>(condition: C) -> impl SystemCondition<(), CIn>
// Fires when wrapped condition output changes (either edge: false→true or true→false)
// Uses Local<bool> for previous-value tracking. Initial assumed previous = false.

pub fn condition_changed_to<Marker, CIn, C>(to: bool, condition: C) -> impl SystemCondition<(), CIn>
// Fires when wrapped condition transitions to `to`.
// Logic: *prev != new && new == to
// Initial assumed previous = false.
```

To detect "SubState S was removed" exactly once on the removal frame:

```rust
.run_if(condition_changed_to(false, state_exists::<S>()))
```

To detect "resource R was just created" exactly once:

```rust
.run_if(condition_changed_to(true, resource_exists::<R>()))
```

---

## Observers Do NOT Support Resources (Bevy 0.18.1)

Verified from `bevy_ecs/src/observer/mod.rs` and `world/mod.rs` at v0.18.1.

Bevy 0.18 Observers are entity/component-only. Built-in trigger types:
- `OnAdd<C>`, `OnInsert<C>`, `OnReplace<C>`, `OnRemove<C>` — component lifecycle on entities
- Custom `Event` types via `world.trigger()` / `commands.trigger()`

`insert_resource()` does NOT fire any observer. Resources have no lifecycle hooks equivalent
to component hooks. There is no `ResourceAdded`, `ResourceMutated`, or similar trigger.

For resource-level reactivity, use `resource_exists_and_changed<T>` (polling) or emit a custom
Message when mutating the resource.

---

## `StateTransitionEvent<S>` as a Message (Bevy 0.18.1)

`StateTransitionEvent<S>` is a `Message` type (not a Bevy `Event`). It fires when any state
transition occurs, including SubState removal (where `entered: None`).

```rust
// Detect NodeState being removed (entered: None case):
.run_if(on_message::<StateTransitionEvent<NodeState>>())
// Then check entered.is_none() in system body to confirm it's a removal
```

This is truly event-driven (zero cost when no transition occurs) and does not require
child cooperation beyond the state machine itself firing the message.

---

## UI Scaling — Val variants, UiScale, TextFont (Bevy 0.18.1)

Verified from: `docs.rs/bevy/0.18.1`, `crates/bevy_ui/src/layout/convert.rs`,
`crates/bevy_ui/src/update.rs`, `crates/bevy_ui/src/widget/text.rs`,
`examples/ui/ui_scaling.rs` all at v0.18.1.

### Val variants (bevy::ui::Val / bevy::prelude::Val)

```rust
pub enum Val {
    Auto,
    Px(f32),        // scaled by: camera.target_scaling_factor() * ui_scale.0
    Percent(f32),   // % of parent node dimension — NOT scaled by UiScale
    Vw(f32),        // % of physical_size.x — NOT scaled by UiScale
    Vh(f32),        // % of physical_size.y — NOT scaled by UiScale
    VMin(f32),      // % of physical_size.min_element() — NOT scaled by UiScale
    VMax(f32),      // % of physical_size.max_element() — NOT scaled by UiScale
}
```

Conversion is in `into_length_percentage_auto` in `convert.rs`:
- `Val::Px(v)` → `scale_factor * v`  (scale_factor = camera factor × ui_scale.0)
- `Val::Vw(v)` → `physical_size.x * v / 100.`  (raw physical pixels, no UiScale)
- `Val::Vh(v)` → `physical_size.y * v / 100.`  (raw physical pixels, no UiScale)

**Gotcha**: Vw/Vh/VMin/VMax respond to window resize automatically (tracked via
`ComputedUiRenderTargetInfo`). Do NOT mix Px and Vw/Vh in the same layout when
UiScale != 1.0 — the two unit systems are on different scales.

### UiScale resource

```rust
// bevy::ui::UiScale (re-exported in bevy::prelude)
pub struct UiScale(pub f32);  // Default: 1.0
```

Applied in `propagate_ui_target_cameras` (`update.rs`):
```
layout_scale_factor = camera.target_scaling_factor() * ui_scale.0
```

**What UiScale scales**: `Val::Px` sizing AND `TextFont::font_size`.
Source comment in `widget/text.rs`: `"scale_factor is already multiplied by UiScale"`.

**What UiScale does NOT scale**: `Val::Vw/Vh/VMin/VMax`, `Val::Percent`.

### TextFont — font_size field

```rust
pub struct TextFont {
    pub font: Handle<Font>,
    pub font_size: f32,  // physical pixels, no viewport-relative variant
    // ...
}
```

No built-in responsive font size in Bevy 0.18.1. Font size IS multiplied by
`scale_factor` (which includes UiScale), so setting UiScale scales fonts globally.

### Recommended pattern: UiScale driven by window dimensions

```rust
fn sync_ui_scale(
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        // Design resolution: 1920×1080. Use min to letterbox.
        let scale = (window.width() / 1920.0).min(window.height() / 1080.0);
        ui_scale.0 = scale;
    }
}
// Run in Update. All Val::Px and font_size designed for 1920×1080 scale automatically.
```

**UiScale is global** — cannot be per-node. If some Px values should not scale
(e.g., hairline 1px borders), use `Val::Px(1.0 / ui_scale.0)` to counteract it,
or switch those values to a different approach.

Full research report: `docs/todos/detail/scenario-runner-verbose-violation-log/research/bevy-ui-scaling.md`
