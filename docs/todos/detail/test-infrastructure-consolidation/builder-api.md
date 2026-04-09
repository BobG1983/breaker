# TestAppBuilder — API Reference & Behavioral Spec

Complete API for the test app builder. Each method includes its signature, behavioral spec (for writing test expectations), and usage example.

## Location

`src/shared/test_utils.rs` — the builder, `MessageCollector<M>`, and `tick()` all live here.

## Core Types

### TestAppBuilder\<S\>

Typestate builder for test `App` instances. Tracks whether the state hierarchy has been registered, preventing compile-time misuse of state-dependent methods like `in_state_node_playing()`.

```rust
pub struct TestAppBuilder<S: StateStatus = NoStates> {
    app: App,
    _state: PhantomData<S>,
}

pub trait StateStatus {}
pub struct NoStates;
pub struct WithStates;
impl StateStatus for NoStates {}
impl StateStatus for WithStates {}
```

### MessageCollector\<M\>

Generic message collector that replaces all per-message collector structs (`DamageCellCollector`, `CapturedBumps`, `HitCells`, etc.).

```rust
#[derive(Resource)]
pub struct MessageCollector<M: Message>(pub Vec<M>);

impl<M: Message> Default for MessageCollector<M> {
    fn default() -> Self { Self(Vec::new()) }
}

impl<M: Message> MessageCollector<M> {
    /// Manually clears collected messages (e.g., mid-tick reset).
    pub fn clear(&mut self) { self.0.clear(); }
}
```

**Auto-cleared every `app.update()`**: A clear system runs in `First` schedule, so each tick's collector contains only that tick's messages. Tests that need accumulation track a running count themselves.

**Test usage**:
```rust
// After one tick — only this tick's messages
tick(&mut app);
let collected = app.world().resource::<MessageCollector<DamageCell>>();
assert_eq!(collected.0.len(), 1);
assert_eq!(collected.0[0].damage, 10.0);

// After another tick — auto-cleared, only new messages
tick(&mut app);
let collected = app.world().resource::<MessageCollector<DamageCell>>();
assert_eq!(collected.0.len(), 0); // no new messages this tick

// Accumulation pattern (when needed)
let mut total = 0;
tick(&mut app);
total += app.world().resource::<MessageCollector<DamageCell>>().0.len();
tick(&mut app);
total += app.world().resource::<MessageCollector<DamageCell>>().0.len();
assert_eq!(total, 3);
```

---

## Construction

### TestAppBuilder::new()

**Signature**: `pub fn new() -> TestAppBuilder<NoStates>`

**Behavior**:
- Given: nothing
- When: `TestAppBuilder::new()` is called
- Then: returns a builder wrapping an `App` with only `MinimalPlugins` registered
- Then: no resources, messages, states, or systems are registered beyond `MinimalPlugins` defaults
- Edge case: calling `.build()` immediately produces a valid minimal `App`

---

## State Hierarchy

### .with_state_hierarchy()

**Signature**: `pub fn with_state_hierarchy(self) -> TestAppBuilder<WithStates>`

**Behavior**:
- Given: a builder in `NoStates`
- When: `.with_state_hierarchy()` is called
- Then: `bevy::state::app::StatesPlugin` is added
- Then: `AppState` is initialized via `init_state`
- Then: `GameState`, `RunState`, `NodeState`, `ChipSelectState`, `RunEndState` are added as sub-states
- Then: the builder transitions to `WithStates` typestate

**Compile-time constraint**: Only available on `TestAppBuilder<NoStates>`. Cannot be called on `WithStates` (prevents double-registration).

---

## State Navigation (requires WithStates)

### .in_state_node_playing()

**Signature**: `pub fn in_state_node_playing(self) -> TestAppBuilder<WithStates>`

**Behavior**:
- Given: a builder with state hierarchy registered
- When: `.in_state_node_playing()` is called
- Then: sets `NextState<AppState>` to `Game`, calls `app.update()`
- Then: sets `NextState<GameState>` to `Run`, calls `app.update()`
- Then: sets `NextState<RunState>` to `Node`, calls `app.update()`
- Then: sets `NextState<NodeState>` to `Playing`, calls `app.update()`
- Then: after build, systems with `run_if(in_state(NodeState::Playing))` will execute

**Compile-time constraint**: Only available on `TestAppBuilder<WithStates>`.

### .in_state_chip_selecting()

**Signature**: `pub fn in_state_chip_selecting(self) -> TestAppBuilder<WithStates>`

**Behavior**:
- Given: a builder with state hierarchy registered
- When: `.in_state_chip_selecting()` is called
- Then: sets `NextState<AppState>` to `Game`, calls `app.update()`
- Then: sets `NextState<GameState>` to `Run`, calls `app.update()`
- Then: sets `NextState<RunState>` to `ChipSelect`, calls `app.update()`
- Then: sets `NextState<ChipSelectState>` to `Selecting`, calls `app.update()`

**Compile-time constraint**: Only available on `TestAppBuilder<WithStates>`.

---

## Plugins

### .with_physics()

**Signature**: `pub fn with_physics(self) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_physics()` is called
- Then: `RantzPhysics2dPlugin` is added
- Then: quadtree, CCD, collision layer systems, and spatial query resources are available

---

## Resource Bundles

### .with_playfield()

**Signature**: `pub fn with_playfield(self) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_playfield()` is called
- Then: `PlayfieldConfig` is initialized with defaults
- Then: `CellConfig` is initialized with defaults
- Then: `Assets<Mesh>` is initialized
- Then: `Assets<ColorMaterial>` is initialized
- Edge case: call `.insert_resource(custom_config)` after `.with_playfield()` to override individual resources (Bevy's `insert_resource` overwrites)

---

## Individual Resources

### .with_resource\<R\>()

**Signature**: `pub fn with_resource<R: Resource + Default>(self) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_resource::<R>()` is called
- Then: resource `R` is initialized with `Default::default()`

### .insert_resource()

**Signature**: `pub fn insert_resource<R: Resource>(self, resource: R) -> Self`

**Behavior**:
- Given: any builder and a concrete resource value
- When: `.insert_resource(value)` is called
- Then: resource is inserted (overwrites if already present)

---

## Messages

### .with_message\<M\>()

**Signature**: `pub fn with_message<M: Message>(self) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_message::<M>()` is called
- Then: message type `M` is registered via `app.add_message::<M>()`
- Then: systems can send and read messages of type `M`

### .with_message_capture\<M\>()

**Signature**: `pub fn with_message_capture<M: Message + Clone + 'static>(self) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_message_capture::<M>()` is called
- Then: message type `M` is registered
- Then: `MessageCollector<M>` resource is initialized (empty `Vec`)
- Then: a `collect_messages::<M>` system is added in the `Last` schedule
- Then: a `clear_messages::<M>` system is added in `First` schedule (auto-clears at start of each `app.update()`)
- Then: a `collect_messages::<M>` system is added in `Last` schedule (captures after all other systems)
- Then: after each `tick()`, `MessageCollector<M>.0` contains only that tick's messages
- Then: `collector.clear()` is available for manual mid-tick resets

**Systems** (registered automatically):
```rust
fn clear_messages<M: Message>(mut collector: ResMut<MessageCollector<M>>) {
    collector.0.clear();
}

fn collect_messages<M: Message + Clone>(
    mut reader: MessageReader<M>,
    mut collector: ResMut<MessageCollector<M>>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}
```

- `clear_messages` runs in `First` — clears before any systems send messages
- `collect_messages` runs in `Last` — captures after all systems have run

---

## Registries

Each registry method pair follows the same pattern: `.with_*_registry()` creates an empty registry, `.with_*_registry_entry()` creates the registry if needed and inserts an entry.

### .with_bolt_registry()

**Signature**: `pub fn with_bolt_registry(self) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_bolt_registry()` is called
- Then: an empty `BoltRegistry` is inserted if not already present

### .with_bolt_registry_entry()

**Signature**: `pub fn with_bolt_registry_entry(self, name: impl Into<String>, def: BoltDefinition) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_bolt_registry_entry(name, def)` is called
- Then: `BoltRegistry` is created if not present
- Then: the definition is inserted under the given name
- Edge case: calling multiple times with different names adds multiple entries
- Edge case: calling with the same name overwrites the previous entry

### .with_breaker_registry() / .with_breaker_registry_entry()

Same pattern as bolt, with `BreakerRegistry` and `BreakerDefinition`.

### .with_cell_registry() / .with_cell_registry_entry()

Same pattern as bolt, with `CellTypeRegistry` and `CellTypeDefinition`.

---

## Systems

### .with_system()

**Signature**: `pub fn with_system<M>(self, schedule: impl ScheduleLabel, system: impl IntoSystemConfigs<M>) -> Self`

**Behavior**:
- Given: any builder
- When: `.with_system(schedule, system)` is called
- Then: the system (or system tuple with ordering) is added to the specified schedule

**Examples**:
```rust
// Single system
.with_system(FixedUpdate, bolt_lost)

// System with ordering
.with_system(FixedUpdate, bolt_cell_collision.after(PhysicsSystems::MaintainQuadtree))

// Multiple systems with ordering
.with_system(FixedUpdate, (
    enqueue_trigger.before(bridge_trigger),
    bridge_trigger,
))
```

---

## Build

### .build()

**Signature**: `pub fn build(self) -> App`

**Behavior**:
- Given: a configured builder
- When: `.build()` is called
- Then: returns the `App` ready for testing
- Then: if state navigation was used, the app is already in the target state
- Then: if no state navigation was used, no `app.update()` has been called

---

## Shared Utility Functions

Standalone functions alongside the builder in `src/shared/test_utils.rs`.

### tick()

**Signature**: `pub(crate) fn tick(app: &mut App)`

**Behavior**:
- Given: an app with `Time<Fixed>` resource (provided by `MinimalPlugins`)
- When: `tick(app)` is called
- Then: one `FixedUpdate` timestep is accumulated via `Time<Fixed>::accumulate_overstep`
- Then: `app.update()` is called, executing all scheduled systems
- Edge case: calling `tick` N times advances N fixed timesteps

### Spawning Entities in Tests

There is no `spawn_in_world` helper. Bevy 0.18 provides `World::commands()` + `World::flush()` natively.

**For builder-based APIs** (Bolt::builder, Breaker::builder, etc.):
```rust
let world = app.world_mut();
let entity = Bolt::builder()
    .at_position(Vec2::new(x, y))
    .definition(&def)
    .with_velocity(Velocity2D(Vec2::new(vx, vy)))
    .primary()
    .headless()
    .spawn(&mut world.commands());
world.flush();
```

**For direct component bundles** (no commands/flush needed):
```rust
let entity = app.world_mut().spawn((Cell, Position2D(pos), Aabb2D::new(...))).id();
```

Domain spawners in `test_utils.rs` (`spawn_bolt()`, `spawn_cell()`, etc.) encapsulate the `commands()` + `flush()` pattern internally. Individual tests use the spawners, never raw commands.
