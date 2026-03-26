---
name: Resource derive and system ordering
description: Resource trait bounds, generic Resource types, .before()/.after()/.in_set() for FixedUpdate ordering
type: reference
---

## Verified against: docs.rs/bevy/0.18.1

---

## `Resource` trait

```rust
pub trait Resource: Send + Sync + 'static {}
```

Path: `bevy::prelude::Resource` (in prelude, also at `bevy::ecs::system::Resource`)

### Deriving Resource on a generic type

Yes — `#[derive(Resource)]` works on generic structs. The derive macro automatically
propagates the `Send + Sync + 'static` bounds to all type parameters.

```rust
#[derive(Resource)]
struct SpatialIndex(Quadtree<Entity>);  // non-generic newtype — simplest

// OR with generic param — T must be Send + Sync + 'static:
#[derive(Resource)]
struct SpatialIndex<T>(Quadtree<T>);
// Derived impl requires: T: Send + Sync + 'static
```

For `SpatialIndex` wrapping `Quadtree<Entity>`: a plain newtype is the cleanest approach
since `Entity: Send + Sync + 'static`. No manual impl needed.

---

## System ordering: `.before()`, `.after()`, `.in_set()`

These methods are on `IntoScheduleConfigs` — available on any system or system tuple via
the `add_systems` call chain.

```rust
app.add_systems(FixedUpdate, my_system.before(OtherPlugin::SomeSet));
app.add_systems(FixedUpdate, my_system.after(PhysicsSet::Step));
app.add_systems(FixedUpdate, my_system.in_set(MySystems::Index));
```

**Cross-plugin sets work**, provided:
1. The external set is scheduled in the **same schedule** (e.g., both in `FixedUpdate`).
   Ordering constraints between different schedules are **silently ignored**.
2. The external set is actually registered in the app — either by a third-party plugin or
   manually. If the set is never scheduled, the constraint has no effect.

### Configuring sets

```rust
app.configure_sets(
    FixedUpdate,
    MySystems::Index.before(PhysicsSet::Collision),
);
```

This is cleaner than adding `.before()` on every individual system.

---

## Ordering for spatial index maintenance

Correct pattern for a `maintain_spatial_index` system in FixedUpdate that must run
BEFORE collision detection:

```rust
// In your plugin:
app.configure_sets(
    FixedUpdate,
    SpatialIndexSystems::Maintain.before(PhysicsSet::Collision),
);
app.add_systems(
    FixedUpdate,
    maintain_spatial_index.in_set(SpatialIndexSystems::Maintain),
);
```

---

## Sources

- Resource trait: https://docs.rs/bevy/0.18.1/bevy/prelude/trait.Resource.html
- IntoScheduleConfigs: https://docs.rs/bevy/0.18.1/bevy/ecs/schedule/trait.IntoScheduleConfigs.html
