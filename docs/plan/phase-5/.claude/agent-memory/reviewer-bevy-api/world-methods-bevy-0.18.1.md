---
name: Confirmed Bevy 0.18.1 World Method Signatures
description: Verified-correct signatures for World direct-access methods used in effect fire() functions
type: reference
---

## Verified for Bevy 0.18.1

### World::get_entity
```rust
pub fn get_entity<F>(&self, entities: F) -> Result<<F as WorldEntityFetch>::Ref<'_>, EntityNotSpawnedError>
```
Returns `Result`, NOT `Option`. `.is_err()` is correct for existence check.

### World::get
```rust
pub fn get<T: Component>(&self, entity: Entity) -> Option<&T>
```
Returns `Option<&T>`. `.is_none()` check correct for component existence.

### World::get_mut
```rust
pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<Mut<'_, T>>
```
Returns `Option<Mut<'_, T>>`. Pattern `if let Some(mut x) = world.get_mut::<T>(entity)` is correct.

### World::entity_mut
```rust
pub fn entity_mut(&mut self, entity: Entity) -> EntityWorldMut<'_>
```
Returns `EntityWorldMut`. `.insert(bundle)` on the return value is correct.

### World::despawn
```rust
pub fn despawn(&mut self, entity: Entity)
```
Returns `()` (unit). Does NOT return bool or Result. Use `world.try_despawn()` for Result. Pattern `world.despawn(entity)` with no return value check is correct.

### World::get_resource_or_insert_with
```rust
pub fn get_resource_or_insert_with<R: Resource>(&mut self, f: impl FnOnce() -> R) -> Mut<'_, R>
```
Returns `Mut<'_, R>` (NOT `&mut R`). The returned `Mut` holds a `&mut World` borrow for its lifetime — MUST be dropped before world is used again. Scoping in blocks `{}` is the correct pattern.

### World::get_resource_mut
```rust
pub fn get_resource_mut<R: Resource>(&mut self, ) -> Option<Mut<'_, R>>
```
Returns `Option<Mut<'_, R>>`. Pattern `if let Some(mut r) = world.get_resource_mut::<T>()` is correct.

### World::query
```rust
pub fn query<Q: QueryData>(&mut self) -> QueryState<Q>
```
Returns owned `QueryState<Q>` by value. Takes `&mut self` but releases borrow on return. Then call `query.iter(&world)` with immutable `&World`.

### QueryState::iter
```rust
pub fn iter<'w, 's>(&'s mut self, world: &'w World) -> QueryIter<...>
```
Takes `&World` (immutable). Requires `&mut self` on the QueryState. No conflict with world re-borrow after `world.query()` returns.

## Confirmed Safe Patterns (used across multiple effect files)

### Stat-boost lazy init pattern
```rust
if world.get_entity(entity).is_err() { return; }
if world.get::<ActiveX>(entity).is_none() {
    world.entity_mut(entity).insert((ActiveX::default(), EffectiveX::default()));
}
if world.get::<EffectiveX>(entity).is_none() {
    world.entity_mut(entity).insert(EffectiveX::default());
}
if let Some(mut active) = world.get_mut::<ActiveX>(entity) {
    active.0.push(value);
}
```
All calls are sequential, no overlapping borrows. CORRECT for 0.18.1.

### FIFO resource + query pattern
```rust
// Scope A: get counter, copy value out, drop Mut borrow
let counter_value: u64 = {
    let counter_resource = world.get_resource_or_insert_with(T::default);
    *counter_resource.0.get(&entity).unwrap_or(&0)
};
// Scope B: query (world borrow released from Scope A)
let to_despawn: Vec<Entity> = {
    let mut query = world.query::<(Entity, &A, &B)>();
    query.iter(world).filter(...).collect()
    // + despawn list logic
};
// Despawn after query scope
for e in &to_despawn { world.despawn(*e); }
// Spawn
world.spawn((components...));
// Scope C: re-borrow resource
if let Some(mut r) = world.get_resource_mut::<T>() {
    r.0.insert(entity, counter_value + 1);
}
```
All borrows are properly scoped. CORRECT for 0.18.1.
