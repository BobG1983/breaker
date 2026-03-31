# Confirmed Bevy 0.18.1 API Patterns

Verified against docs.rs/bevy/0.18.1 and official source.

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

## Message System (Bevy 0.18 observer pattern)

TODO: add from next research session.

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

Fields can themselves be another `#[derive(QueryData)]` struct — no special syntax needed:

```rust
#[derive(QueryData)]
struct Inner { c: &'static ComponentC }

#[derive(QueryData)]
#[query_data(mutable)]
struct Outer {
    a: &'static mut ComponentA,
    inner: Inner,  // nested read-only QueryData
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

### Key rules summary

1. Any `&'static mut` field → **requires** `#[query_data(mutable)]` on the struct
2. Read-only fields (`&'static T`) are valid in a mutable struct
3. Field type is always `&'static T` or `&'static mut T` (with the `'static` lifetime) in the struct definition
4. The actual item type in system iteration uses short lifetimes (`'w`) — Bevy handles this
5. Tuple fields work: `Option<(&'static ComponentB, &'static ComponentZ)>`

---

## State

TODO: add from next research session.

---

## SystemParam derive

TODO: add from next research session.

---

## Component spawning

TODO: add from next research session.

---

## World access

TODO: add from next research session.

---

## Let-chains

TODO: add from next research session.
