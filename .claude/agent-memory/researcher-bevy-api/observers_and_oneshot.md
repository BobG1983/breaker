# Observers, Triggers, One-Shot Systems — Bevy 0.18.1

Verified against: docs.rs/bevy/0.18.1, bevy.org/news/bevy-0-14/, github.com/bevyengine/bevy v0.18.0 examples/ecs/observers.rs

---

## Observers

Observers are push-based, immediate, synchronous. They are ECS entities that hold a system.

### Defining an observed event

```rust
#[derive(Event)]
struct MyEvent { data: f32 }

// Entity-targeted variant
#[derive(EntityEvent)]
struct MyEntityEvent { value: i32 }
```

`Event` trait: `pub trait Event: Sized + Send + Sync + 'static { type Trigger<'a>: Trigger<Self>; }`
Default derive uses `GlobalTrigger` (fires to all observers).
`EntityEvent` uses `EntityTrigger` (fires to observers watching that entity).

### Registering observers

**App-level (global) — runs for ALL firings of that event:**
```rust
app.add_observer(|e: On<MyEvent>, ...| { /* ... */ });
```

**World-level (global):**
```rust
world.add_observer(|e: On<MyEvent>| { ... }); // returns EntityWorldMut
```

**Entity-specific — runs only when that entity is targeted:**
```rust
commands.entity(id).observe(|e: On<MyEntityEvent>| { ... });
// OR at spawn time:
commands.spawn((MyComponent,)).observe(my_handler);
```

**Manual Observer with multiple entity targets (builder style):**
```rust
let obs = Observer::new(my_handler)
    .with_entities([entity_a, entity_b]);
commands.spawn(obs);
// Note: with_entity/with_entities MUST be called before spawn — cannot retarget after spawn
```

`Observer::watch_entity(&mut self, entity: Entity)` — mutable post-build, but has NO EFFECT after spawning.

### On<E> system parameter

```rust
pub struct On<'w, 't, E: Event, B: Bundle = ()>
```

- `e.event()` — `&E`
- `e.event_mut()` — `&mut E`
- `e.observer()` — `Entity` (the observer entity itself)
- `e.propagate(bool)` / `e.get_propagate()` — propagation control (EntityEvent only)
- Implements `Deref<Target=E>` and `DerefMut` — so `e.my_field` works directly

### Triggering (firing) an observer event

**From `Commands` (deferred — runs when commands flush):**
```rust
commands.trigger(MyEvent { data: 1.0 });
commands.trigger_targets(MyEntityEvent { value: 5 }, entity_id);
commands.trigger_targets(MyEntityEvent { value: 5 }, [entity_a, entity_b]);
```

**From `World` (immediate — runs before call returns):**
```rust
world.trigger(MyEvent { data: 1.0 });
// trigger_with if you need a custom Trigger impl
```

**Timing critical distinction:**
- `world.trigger()` = synchronous, observers run before the call returns
- `commands.trigger()` = deferred, observers run at next command flush (end of system)

### Observer execution model

- Observers are NOT part of the ECS schedule. They run outside the schedule, immediately.
- Hooks run before observers.
- Observers CAN chain: an observer can `commands.trigger()` another event. Entire chain evaluates as one transaction when the first event fires.
- Multiple observers can watch the same event — all run, in registration order.
- Observers DO support parallelism? NO — observers run synchronously in sequence.

### Dynamic observer registration at runtime

YES — observers are just entities. Spawn them any time:
```rust
commands.spawn(Observer::new(my_handler).with_entity(some_entity));
```
This works mid-game, mid-frame (deferred to command flush). No rebuild step required.

### Observers vs schedule systems

| Property | Observer | Schedule System |
|----------|----------|-----------------|
| Timing | Immediate (or at cmd flush) | Once per frame in schedule |
| Parallelism | Sequential | Can parallelize |
| Trigger | explicit `.trigger()` call | runs every tick |
| Data delivery | via On<E> param | via MessageReader / Query |
| Multiple readers | All observers run per firing | Yes (MessageReader) |

---

## One-Shot Systems

One-shot systems are registered once and run on demand, not on a schedule.

### Registration

```rust
// On World directly:
let id: SystemId = world.register_system(my_system_fn);

// Cached (zero-sized system fn only — re-registers same fn returns same id):
let id: SystemId = world.register_system_cached(my_system_fn);
```

`SystemId<I = (), O = ()>` — copy type, keyed to a specific World.
`SystemId::entity(self) -> Entity` — underlying entity.
`SystemId::from_entity(entity: Entity) -> SystemId<I, O>`

### Running

```rust
// No input:
world.run_system(id)?;

// With input:
world.run_system_with(id, input_value)?;

// Cached (register+run in one):
world.run_system_cached(my_system_fn)?;
world.run_system_cached_with(my_system_fn, input)?;
```

From `Commands`:
```rust
commands.run_system_cached(my_system_fn); // output = ()
```

### Can systems be added to schedules at runtime?

Technically `Schedule::add_systems()` exists and `schedule.initialize()` rebuilds the executor.
HOWEVER: this is NOT the intended pattern and requires exclusive World access.
Preferred runtime-dynamic pattern: register one-shot systems at startup, run them via `SystemId` on demand.

---

## Component Lifecycle Hooks (OnAdd/OnInsert/OnRemove)

These are NOT observers — they are lower-level hooks defined on the component type itself.

```rust
fn on_add() -> Option<for<'w> fn(DeferredWorld<'w>, HookContext)>
fn on_insert() -> Option<for<'w> fn(DeferredWorld<'w>, HookContext)>
fn on_replace() -> Option<for<'w> fn(DeferredWorld<'w>, HookContext)>
fn on_remove() -> Option<for<'w> fn(DeferredWorld<'w>, HookContext)>
fn on_despawn() -> Option<for<'w> fn(DeferredWorld<'w>, HookContext)>
```

Via derive macro:
```rust
#[derive(Component)]
#[component(on_add = Self::my_hook)]
struct MyComp;
impl MyComp {
    fn my_hook(world: DeferredWorld, ctx: HookContext) { ... }
}
```

`HookContext` gives the entity that triggered the lifecycle event.
ONE hook per lifecycle event per component type — cannot be overridden.
Hooks run BEFORE observers.

There are also built-in `Add<C>`, `Insert<C>`, `Remove<C>` trigger events that observers can watch:
```rust
app.add_observer(|e: On<Add, MyComp>| { ... });
app.add_observer(|e: On<Remove, MyComp>| { ... });
```
These let MULTIPLE observers watch the same lifecycle event (unlike hooks which allow only one).

---

## Messages vs Events (project-specific)

This project uses `#[derive(Message)]` for game communication, NOT `#[derive(Event)]`.
`#[derive(Event)]` is ONLY for observer-triggered events in this project.

Key differences:
- Messages: pull-based, multiple independent readers with cursors, buffered per frame
- Events (observers): push-based, immediate, all matching observers run synchronously
- MessageReader maintains per-system cursor — same message readable by N systems
- No built-in run condition for "message available" — check `.read().next().is_some()` or just iterate

---

## Sources

- Observer module: https://docs.rs/bevy/0.18.1/bevy/ecs/observer/index.html
- Observer struct: https://docs.rs/bevy/0.18.1/bevy/ecs/observer/struct.Observer.html
- On param: https://docs.rs/bevy/0.18.1/bevy/ecs/observer/struct.On.html
- Event trait: https://docs.rs/bevy/0.18.1/bevy/ecs/event/trait.Event.html
- SystemId: https://docs.rs/bevy/0.18.1/bevy/ecs/system/struct.SystemId.html
- World methods: https://docs.rs/bevy/0.18.1/bevy/ecs/world/struct.World.html
- Component hooks: https://docs.rs/bevy/0.18.1/bevy/ecs/component/trait.Component.html
- Observer timing: https://bevy.org/news/bevy-0-14/ (hooks and observers section)
- Observers example: https://github.com/bevyengine/bevy/blob/v0.18.0/examples/ecs/observers.rs
