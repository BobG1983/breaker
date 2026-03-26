---
name: Change and Removal Detection APIs
description: Added<T>, Changed<T>, Ref<T>, RemovedComponents<T> — verified Bevy 0.18.1 signatures and behavior
type: reference
---

## Verified against: docs.rs/bevy/0.18.1, bevy prelude page

---

## `Added<T>` — query filter

```rust
pub struct Added<T>(/* private fields */);  // T: Component
```

Path: `bevy::prelude::Added` (in prelude)

- Retains results **only the first time** after the component is added to an entity.
- **Fires on initial insertion** — yes. Also fires for additions that happened before the first time
  the query ran (i.e., catches up on startup).
- Does NOT fire again on subsequent mutations — use `Changed<T>` for that.
- NOT an `ArchetypeFilter` — iterates all matching entities even if none were just added.
  For a million-entity query this can be expensive.

Usage:
```rust
fn on_new_aabb(query: Query<(Entity, &Aabb2d, &CollisionLayers), Added<Aabb2d>>) {
    for (entity, aabb, layers) in &query {
        // runs only on the frame the Aabb2d component was first inserted
    }
}
```

---

## `Changed<T>` — query filter

```rust
pub struct Changed<T>(/* private fields */);  // T: Component
```

Path: `bevy::prelude::Changed` (in prelude)

- Retains results the first time after a component is **added OR mutably dereferenced**.
- **Fires on initial insertion** — yes (same as Added).
- Fires again any time the component is accessed via `&mut` or `DerefMut` — even if the
  value didn't actually change. Bevy does NOT deep-compare values.
- NOT an `ArchetypeFilter` — same performance note as Added<T>.

Filtering by change to a *different* component than you're reading:
```rust
fn on_transform_changed(query: Query<&Name, Changed<Transform>>) {
    for name in &query {
        // entity's Transform was added or mutated; we're reading Name
    }
}
```

---

## `Ref<T>` — query data (access + change detection in one)

```rust
pub struct Ref<'w, T: Component>  // implements Deref<Target=T>, ReadOnlyQueryData
```

Path: `bevy::prelude::Ref` (in prelude)

Methods (from `DetectChanges` trait):
- `is_added() -> bool`  — true if added after the system last ran
- `is_changed() -> bool` — true if added or mutated since last run
- `last_changed() -> Tick` — tick when last changed

Usage — preferred when you need both the value and change detection in one query item:
```rust
fn sync_index(query: Query<(Entity, Ref<Aabb2d>)>) {
    for (entity, aabb) in &query {
        if aabb.is_added() {
            // insert into index
        } else if aabb.is_changed() {
            // update in index
        }
    }
}
```

---

## `RemovedComponents<T>` — system parameter

```rust
pub struct RemovedComponents<'w, 's, T: Component>
```

Path: `bevy::prelude::RemovedComponents` (in prelude)

### What it detects

Fires when:
- A component `T` is explicitly removed from an entity (via `commands.entity(e).remove::<T>()`)
- An entity that had `T` is despawned

Both removal and despawn yield the entity ID via `.read()`.

### Methods

```rust
// Primary iteration — call this to get removed entity IDs:
pub fn read(&mut self) -> impl Iterator<Item = Entity>

// Iteration with message IDs (rarely needed):
pub fn read_with_id(&mut self) -> impl Iterator<Item = (Entity, MessageId<RemovedComponentEntity>)>

// Diagnostics:
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn clear(&mut self)

// Low-level cursor access (not normally needed):
pub fn reader(&self) -> &MessageCursor<RemovedComponentEntity>
pub fn reader_mut(&mut self) -> &mut MessageCursor<RemovedComponentEntity>
pub fn messages(&self) -> Option<&Messages<RemovedComponentEntity>>
```

### Usage

```rust
fn remove_from_index(
    mut removed: RemovedComponents<Aabb2d>,
    mut index: ResMut<SpatialIndex>,
) {
    for entity in removed.read() {
        index.remove(entity);
    }
}
```

**CRITICAL**: must be `&mut self` on `removed` to advance the cursor — declare the parameter
as `mut removed: RemovedComponents<Aabb2d>`.

### Gotchas

- The cursor is per-system, like a `MessageReader`. If you don't call `.read()` the cursor
  doesn't advance — events do NOT auto-clear.
- `clear()` discards all pending removals for this system's cursor (use with care).
- Entity IDs returned may be **recycled** by the time the system runs if a new entity was
  spawned in the same frame. Always treat removed IDs as "remove from your data structure"
  only — do not query for their components after removal.

---

## Sources

- Added: https://docs.rs/bevy/0.18.1/bevy/ecs/query/struct.Added.html
- Changed: https://docs.rs/bevy/0.18.1/bevy/ecs/query/struct.Changed.html
- Ref: https://docs.rs/bevy/0.18.1/bevy/ecs/change_detection/struct.Ref.html
- RemovedComponents: https://docs.rs/bevy/0.18.1/bevy/prelude/struct.RemovedComponents.html
