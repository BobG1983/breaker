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
