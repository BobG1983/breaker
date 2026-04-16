---
name: Bevy 0.18.1 QueryData derive
description: Custom named query structs — read-only, mutable, nested, Has/With distinction, generics, item types
type: reference
---

# Queries — QueryData Derive (Bevy 0.18.1)

Verified against `docs.rs/bevy/0.18.0` and `github.com/bevyengine/bevy/blob/v0.18.0/examples/ecs/custom_query_param.rs`.

## Read-only struct (no mutable components)

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

## Mutable struct (at least one `&'static mut` field)

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

## Generated types (for a struct named `MyQuery`)

| Type | When generated | Use |
|------|---------------|-----|
| `MyQueryItem<'w, 's>` | Always | The item when iterating `&mut query` |
| `MyQueryReadOnly` | Only with `mutable` | Read-only variant usable in `Query<MyQueryReadOnly>` |
| `MyQueryReadOnlyItem<'w, 's>` | Only with `mutable` | The item when iterating `&query` |

## Iterating in a system

```rust
// Immutable iteration — yields read-only items
fn my_system(query: Query<MyQuery>) {
    for item in &query {
        let _ = item.entity;
        let _ = item.position;
    }
}

// Mutable iteration — yields mutable items
fn my_mut_system(mut query: Query<MyQuery>) {
    for mut item in &mut query {
        item.position.translation.x += 1.0;
    }
}
```

## Deriving Debug on the generated item types

```rust
#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
struct MyQuery { ... }
```

## Nesting QueryData structs

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

- Mutable item struct (`OuterItem`): `inner` field resolves to `InnerItem` — which has `Mut<T>` fields.
- Read-only item struct (`OuterReadOnlyItem`): applies `<Inner as QueryData>::ReadOnly`, resolving to `InnerReadOnly`.

**Nesting a read-only struct inside a mutable one** (also valid):

```rust
#[derive(QueryData)]
struct Inner { c: &'static ComponentC }  // no mutable attribute

#[derive(QueryData)]
#[query_data(mutable)]
struct Outer {
    a: &'static mut ComponentA,
    inner: Inner,  // nested read-only QueryData — always read-only
}
```

## Generics

Supported — type parameters must be `Component`:

```rust
#[derive(QueryData)]
struct GenericQuery<T: Component, P: Component> {
    t: &'static T,
    p: &'static P,
}
```

## `Option<>` fields

Supported for optional components, exactly like tuple queries:

```rust
speed_boost: Option<&'static EffectStack<SpeedBoostConfig>>,
```

## Import path

```rust
use bevy::ecs::query::QueryData;
// or via prelude if re-exported (verify — safe to use full path)
```

## `With<T>` CANNOT be a QueryData field — filter only

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

## `Has<T>` — presence check that IS valid inside QueryData

`Has<T>` implements `QueryData` (not `QueryFilter`). It resolves to `bool` — true if the
entity has component `T`, false otherwise.

```rust
#[derive(QueryData)]
#[query_data(mutable)]
struct MyQuery {
    position: &'static mut Transform,
    has_boost: Has<SpeedBoost>,  // resolves to bool — valid QueryData field
}
```

Use `Has<T>` when you want to check component presence in the data struct itself.
Use `With<T>` / `Without<T>` as a filter when you only want to match entities that have/lack the component.

## Key rules summary

1. Any `&'static mut` field → **requires** `#[query_data(mutable)]` on the struct
2. Read-only fields (`&'static T`) are valid in a mutable struct
3. Field type is always `&'static T` or `&'static mut T` (with the `'static` lifetime) in the struct definition
4. The actual item type in system iteration uses short lifetimes (`'w`) — Bevy handles this
5. Tuple fields work: `Option<(&'static ComponentB, &'static ComponentZ)>`
6. `With<T>` is a filter ONLY — not valid as a QueryData field. Use `Query<D, With<T>>` instead.
7. `Has<T>` IS valid as a QueryData field — resolves to `bool` at query time.
