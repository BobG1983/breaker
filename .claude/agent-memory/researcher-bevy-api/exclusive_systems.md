---
name: Exclusive Systems — Bevy 0.18.1
description: Exclusive system signatures, World query API, SystemState pattern, ordering, performance, deferred alternatives
type: reference
---

# Exclusive Systems in Bevy 0.18.1

Verified against: docs.rs/bevy/0.18.1, github.com/bevyengine/bevy v0.18.1 examples/ecs/ecs_guide.rs

---

## 1. Definition and Registration

An exclusive system is any function whose first parameter is `&mut World`. No special marker or wrapper needed.

```rust
fn my_exclusive_system(world: &mut World) {
    // full world access
}

// Registration — identical to any other system:
app.add_systems(FixedUpdate, my_exclusive_system);
app.add_systems(FixedUpdate, my_exclusive_system.in_set(MySystems::MySet));
app.add_systems(FixedUpdate, my_exclusive_system.after(other_system));
```

---

## 2. ExclusiveSystemParam — optional extra parameters after &mut World

Types implementing `ExclusiveSystemParam` can follow `&mut World`:

```rust
fn exclusive_with_state(world: &mut World, state: &mut SystemState<(Query<&Position2D>, ResMut<MyRes>)>) {
    let (query, mut res) = state.get_mut(world);
    // ...
    state.apply(world); // flush Commands if any
}
```

Types that implement `ExclusiveSystemParam`:
- `&mut SystemState<P>` — wraps any normal SystemParam (most important)
- `&mut QueryState<D, F>` — direct query state (lower-level)
- `Local<'_, T>` — persistent local state across invocations
- `PhantomData<S>` — zero-cost marker
- Tuples of any of the above (up to 16)

**Critical: `SystemState` must be cached.** If `SystemState` is declared as an `ExclusiveSystemParam`, Bevy caches it automatically between calls. Do NOT create `SystemState::new()` inside the system body on every call — that resets change detection cursors.

---

## 3. Direct World Query API

### Creating a QueryState

```rust
// Read-only query
let mut qs: QueryState<(&Position2D, &Velocity2D)> = world.query::<(&Position2D, &Velocity2D)>();

// Filtered query — With<T>, Without<T>, etc.
let mut qs: QueryState<&Position2D, With<Bolt>> =
    world.query_filtered::<&Position2D, With<Bolt>>();
```

Signatures:
```rust
pub fn query<Q: WorldQuery>(&mut self) -> QueryState<Q, ()>
pub fn query_filtered<Q: WorldQuery, F: QueryFilter>(&mut self) -> QueryState<Q, F>
```

Both take `&mut self` — they update archetype metadata in the world.

### Using QueryState — key method signatures

```rust
// Read-only iteration — takes &World (can borrow immutably after getting the QueryState)
pub fn iter<'w, 's>(&'s mut self, world: &'w World) -> QueryIter<'w, 's, D::ReadOnly, F>

// Mutable iteration — takes &mut World
pub fn iter_mut<'w, 's>(&'s mut self, world: &'w mut World) -> QueryIter<'w, 's, D, F>

// Read-only single
pub fn single<'w>(&mut self, world: &'w World) -> Result<D::ReadOnly::Item<'w, '_>, QuerySingleError>

// Mutable single
pub fn single_mut<'w>(&mut self, world: &'w mut World) -> Result<D::Item<'w, '_>, QuerySingleError>

// Read-only by entity
pub fn get<'w>(&mut self, world: &'w World, entity: Entity) -> Result<D::ReadOnly::Item<'w, '_>, QueryEntityError>

// Mutable by entity
pub fn get_mut<'w>(&mut self, world: &'w mut World, entity: Entity) -> Result<D::Item<'w, '_>, QueryEntityError>
```

**Borrow splitting gotcha**: `world.query()` takes `&mut self`, but the returned `QueryState` no longer borrows `world`. So you CAN call `query_state.iter(world)` afterward — the borrow of world for iter() is separate from the &mut self for query(). However, you cannot hold two `&mut World` borrows at once, so you cannot run two mutable queries simultaneously without reborrowing in sequence.

### Practical pattern for multiple queries in exclusive system

```rust
fn my_exclusive(world: &mut World) {
    // Step 1: collect what you need from read-only queries
    let mut bolt_positions: Vec<(Entity, Vec2)> = Vec::new();
    {
        let mut qs = world.query_filtered::<(Entity, &Position2D), With<Bolt>>();
        for (e, pos) in qs.iter(world) {
            bolt_positions.push((e, pos.0));
        }
    }

    // Step 2: apply mutations using entity_mut or commands
    for (entity, _pos) in &bolt_positions {
        if let Ok(mut emut) = world.get_entity_mut(*entity) {
            emut.insert(SomeComponent);
        }
    }
}
```

---

## 4. Resource Access from &mut World

```rust
world.resource::<MyRes>()          // &MyRes — panics if missing
world.resource_mut::<MyRes>()      // Mut<MyRes> — panics if missing
world.get_resource::<MyRes>()      // Option<&MyRes>
world.get_resource_mut::<MyRes>()  // Option<Mut<MyRes>>
```

---

## 5. Entity Mutations from &mut World

Direct (immediate, not deferred):
```rust
world.spawn(bundle)                          // -> EntityWorldMut
world.entity_mut(entity)                    // -> EntityWorldMut — panics if missing
world.get_entity_mut(entity)                // -> Result<EntityWorldMut, ...>

// EntityWorldMut methods:
emut.insert(component);
emut.remove::<C>();
emut.despawn();
```

Deferred via Commands:
```rust
let mut commands = world.commands();   // -> Commands<'_, '_>
commands.spawn(bundle);
commands.entity(e).insert(component);
commands.entity(e).remove::<C>();
commands.entity(e).despawn();
// Must flush explicitly:
world.flush();
```

---

## 6. Commands from &mut World

```rust
pub fn commands(&mut self) -> Commands<'_, '_>
```

Returns a `Commands` that writes to the world's internal command queue.
Changes are NOT applied until `world.flush()` is called explicitly.

```rust
fn exclusive(world: &mut World) {
    let mut cmds = world.commands();
    cmds.spawn(MyBundle::default());
    cmds.entity(some_entity).insert(Tag);
    drop(cmds); // drop the borrow so we can call flush
    world.flush();
}
```

Custom command via `queue()`:
```rust
pub fn queue<C, T>(&mut self, command: C)
where C: Command<T> + HandleError<T>
```

Accepts: closures `|world: &mut World| { ... }`, or any struct implementing `Command`.

---

## 7. Ordering — .before() / .after() / .in_set() / .run_if()

**All work identically for exclusive systems as for normal systems.**

Exclusive systems implement `IntoSystemConfigs` just like normal systems. No restrictions.

```rust
app.add_systems(
    FixedUpdate,
    (
        (new_round_system, new_player_system).chain(),
        exclusive_player_system,
    )
    .in_set(MySystems::BeforeRound)
);

app.add_systems(FixedUpdate,
    my_exclusive
        .after(OtherSystems::SomeSet)
        .run_if(in_state(PlayingState::Active))
);
```

---

## 8. Performance — Parallelism Impact

**Exclusive systems block all parallel execution** of other systems in the same schedule frame while they run. The Bevy ecs_guide.rs example explicitly warns:

> "These will block all parallel execution of other systems until they finish, so they should generally be avoided if you want to maximize parallelism."

This means: all systems that could run in parallel on other threads are stalled until the exclusive system completes.

---

## 9. SystemState Pattern — The Preferred "Exclusive + Normal Params" Pattern

`SystemState<P>` lets you use normal `SystemParam` types (Query, Commands, Res, MessageReader, etc.) inside an exclusive system, with full change detection:

```rust
fn effect_evaluator(
    world: &mut World,
    state: &mut SystemState<(
        Query<(&EffectChain, &ArmedEffects)>,
        Query<&Position2D>,
        ResMut<SomeResource>,
        MessageReader<SomeTrigger>,
    )>,
) {
    let (effect_query, pos_query, mut resource, reader) = state.get_mut(world);
    // ... collect what you need
    state.apply(world); // flush Commands
}
```

**Registration of exclusive system with SystemState param:**
```rust
// Bevy caches the SystemState automatically when declared as a param
app.add_systems(FixedUpdate, effect_evaluator);
```

**IMPORTANT**: `SystemState` as an `ExclusiveSystemParam` is cached by Bevy between calls — change detection (Added, Changed) and MessageReader cursors work correctly. If you instead call `SystemState::new(&mut world)` inside the function body each frame, those cursors reset every frame, breaking change detection.

---

## 10. The "Collect then Mutate" Pattern (Alternative to Exclusive)

For the specific use case of "read some components, decide what to do, then mutate" — you can avoid exclusive systems entirely with a normal system that collects work into a buffer:

```rust
fn evaluate_effects(
    effect_query: Query<(&EffectChain, &ArmedEffects)>,
    pos_query: Query<&Position2D>,
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let mut work: Vec<(Entity, EffectAction)> = Vec::new();

    for (chain, armed) in &effect_query {
        // read-only phase: collect what needs to happen
        // ...
        work.push((entity, action));
    }

    for (entity, action) in work {
        // mutation phase: apply via commands or message writers
        match action {
            EffectAction::Spawn(bundle) => { commands.spawn(bundle); }
            EffectAction::Insert(e, c) => { commands.entity(e).insert(c); }
            EffectAction::Damage(msg) => { damage_writer.write(msg); }
        }
    }
}
```

This works well as long as:
- You don't need to read-then-immediately-mutate in the same frame tick and then re-read the mutations
- Effect logic doesn't need to branch on the results of mutations it just issued
- You can use Commands for structural changes (they flush at end-of-system or at apply_deferred)

---

## 11. run_system_once — For Tests and One-Off Execution

```rust
fn run_system_once<T, Out, Marker>(self, system: T) -> Result<Out, RunSystemError>
where T: IntoSystem<(), Out, Marker>
```

Implemented on `&mut World` via the `RunSystemOnce` trait. Local variables reset every call, change detection does not persist. Suitable for tests — not for production gameplay logic that needs state.

---

## Sources

- World struct: https://docs.rs/bevy/0.18.1/bevy/ecs/world/struct.World.html
- ExclusiveSystemParam: https://docs.rs/bevy/0.18.1/bevy/ecs/system/trait.ExclusiveSystemParam.html
- SystemState: https://docs.rs/bevy/0.18.1/bevy/ecs/system/struct.SystemState.html
- QueryState: https://docs.rs/bevy/0.18.1/bevy/ecs/query/struct.QueryState.html
- RunSystemOnce: https://docs.rs/bevy/0.18.1/bevy/ecs/system/trait.RunSystemOnce.html
- ecs_guide.rs example: https://github.com/bevyengine/bevy/blob/v0.18.1/examples/ecs/ecs_guide.rs
