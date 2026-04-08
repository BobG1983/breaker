---
name: Bevy 0.18.1 Run Conditions
description: Run condition combinators (.and/.or/.nand/.nor), resource change detection, Observers limitation
type: reference
---

# Run Condition Combinators (Bevy 0.18.1)

Verified from `github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_ecs/src/schedule/condition.rs`.

All run conditions implement `SystemCondition` (an alias for `IntoSystem<In, bool, Marker>` where
the system is `ReadOnlySystem`). The trait provides combinator methods:

```rust
fn and<M, C: SystemCondition<M, In>>(self, and: C) -> And<Self::System, C::System>
fn or<M, C: SystemCondition<M, In>>(self, or: C) -> Or<Self::System, C::System>
fn nand<M, C: SystemCondition<M, In>>(self, nand: C) -> Nand<Self::System, C::System>
fn nor<M, C: SystemCondition<M, In>>(self, nor: C) -> Nor<Self::System, C::System>
```

`.and()` short-circuits: if left is false, right is never evaluated.
`.or()` short-circuits: if left is true, right is never evaluated.

```rust
// Example: fire only when message arrived AND in a specific state
.run_if(on_message::<NodeExited>().and(in_state(RunState::Node)))
```

---

# Resource Change Detection Run Conditions (Bevy 0.18.1)

Verified from source and docs.rs/bevy/0.18.1.

## `resource_changed<T>` — panics if resource absent

```rust
pub fn resource_changed<T>(res: Res<'_, T>) -> bool where T: Resource
// Implementation: res.is_changed()
// PANICS if T does not exist in the world
```

## `resource_exists_and_changed<T>` — safe variant

```rust
pub fn resource_exists_and_changed<T>(res: Option<Res<'_, T>>) -> bool where T: Resource
// Implementation: match res { Some(r) => r.is_changed(), None => false }
// Returns false (no panic) if T does not exist
```

"Changed" means "mutably dereferenced since the condition last ran" — Bevy does not compare
values. `ResMut<T>` access sets changed even if no mutation occurred.

## `resource_changed_or_removed<T>` — detects removal too

```rust
pub fn resource_changed_or_removed<T>(
    res: Option<Res<'_, T>>,
    existed: Local<'_, bool>,
) -> bool where T: Resource
// Returns false if resource does not exist (uses Option)
```

## `Res<T>` implements `DetectChanges`

`Res<T>` directly exposes: `is_changed() -> bool`, `is_added() -> bool`, `last_changed() -> Tick`,
`added() -> Tick`. `Ref<T>` is for COMPONENTS only — do NOT use `Ref` for resources.

---

# Observers Do NOT Support Resources (Bevy 0.18.1)

Verified from `bevy_ecs/src/observer/mod.rs` and `world/mod.rs` at v0.18.1.

Bevy 0.18 Observers are entity/component-only. Built-in trigger types:
- `OnAdd<C>`, `OnInsert<C>`, `OnReplace<C>`, `OnRemove<C>` — component lifecycle on entities
- Custom `Event` types via `world.trigger()` / `commands.trigger()`

`insert_resource()` does NOT fire any observer. Resources have no lifecycle hooks equivalent
to component hooks. There is no `ResourceAdded`, `ResourceMutated`, or similar trigger.

For resource-level reactivity, use `resource_exists_and_changed<T>` (polling) or emit a custom
Message when mutating the resource.
