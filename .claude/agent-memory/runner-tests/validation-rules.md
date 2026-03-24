---
name: Validation Rules
description: Bevy 0.18.1 API facts relevant to test validation and known build issues
type: reference
---

## Bevy 0.18.1 API Facts
- MessageWriter uses `.write()` method, not `.send()`
- Fixed across: bump.rs, bolt_breaker_collision.rs, bolt_cell_collision.rs, bolt_lost.rs
- Camera: `hdr` field removed — use `Camera::default()` without hdr setting
- App resource access: use `app.world_mut().resource_mut::<T>()`, not `app.world_resource_mut::<T>()`
- Bundle tuple limit: max 15 elements per tuple. A 16-element spawn tuple triggers `E0277 "not a Bundle"`. Fix by nesting sub-tuples or extracting a named `#[derive(Bundle)]` struct.
- `Entity::from_raw(u32)` was removed in Bevy 0.18.1. `Entity::from_index` takes `EntityIndex` (a `NonMaxU32` newtype), NOT a bare `u32` — passing a plain integer literal causes E0308. In test code creating a dummy/stale entity, use `Entity::from_index(EntityIndex::new(9999))` and import `use bevy::ecs::entity::EntityIndex;`. Other constructors: `Entity::from_raw_u32(u32) -> Option<Entity>`, `Entity::from_bits(u64)`, `Entity::from_index_and_generation(index, generation)`.
- `app.world().iter()` / `.world_mut()` borrow conflict in tests: calling `.iter(app.world())` then `.world_mut()` within the same expression scope triggers E0502. Separate borrows into distinct `let` bindings before the mutable borrow.
- `#[require(T(|| expr))]` with named-field structs: Bevy's require macro expands `T(|| expr)` to `T(expr)`, which is tuple-struct construction syntax. If T is a named-field struct (`struct T { a: u32, b: u32 }`), this produces E0423 "expected function, tuple struct or tuple variant, found struct". Fix: make T a tuple struct, or use a separate impl that provides `Default` with the desired values, or use a newtype wrapper that is a tuple struct.
