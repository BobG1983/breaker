---
name: Additional confirmed Bevy 0.18.1 patterns
description: Messages::drain(), iter_current_update_messages(), get_resource_or_insert_with, Query::single_mut() Result return, NodeUi::UiPass anchor verified correct
type: reference
---

# Additional Confirmed Bevy 0.18.1 Patterns

## Messages<T> methods (verified docs.rs/bevy_ecs/0.18.1)

- `Messages<T>::drain()` — exists, draining iterator that removes all messages. Used by orchestrate_transitions to consume internal lifecycle messages without a cursor.
- `Messages<T>::iter_current_update_messages()` — exists, iterates messages since last update. Used in tests.
- `Messages<T>::write(msg)` — exists, writes to current buffer.

## World::get_resource_or_insert_with (verified docs.rs/bevy/0.18.1)

```rust
pub fn get_resource_or_insert_with<R: Resource>(
    &mut self,
    f: impl FnOnce() -> R,
) -> Mut<'_, R>
```

Valid in Bevy 0.18.1. Used in spawn_phantom effect.rs to lazily initialize PhantomSpawnCounter.

## Query::single_mut() (Bevy 0.18)

Returns `Result<QueryItem, QuerySingleError>` — must be handled with `if let Ok(...)` or `let Ok(...) else`.
The project uses `if let Ok(mut window) = query.single_mut()` which is correct.

## NodeUi::UiPass — post-UI anchor (VERIFIED CORRECT for this branch)

```rust
use bevy::ui_render::graph::NodeUi;

fn node_edges() -> Vec<InternedRenderLabel> {
    vec![
        NodeUi::UiPass.intern(),
        TransitionLabel.intern(),
        Node2d::Upscaling.intern(),
    ]
}
```

This places the transition effect AFTER the UI pass — correct for a screen overlay that should cover everything including game UI. Previously was `Node2d::Tonemapping` (placed under UI). The change to `NodeUi::UiPass` is intentional and API-correct.

## Option<Res<T>> as SystemParam field (verified by project-wide usage)

- `Option<Res<'w, T>>` is a valid `SystemParam` and valid as a `#[derive(SystemParam)]` struct field
- Returns `None` when the resource is not present in the world — correct for optional resources
- Used extensively in this project: `CellSpawnContext`, `debug_ui`, `init_node_timer`, etc.
- Also valid as a direct system parameter: `fn advance_node(sequence: Option<Res<NodeSequence>>)`
- `.as_deref()` converts `Option<Res<T>>` → `Option<&T>` — correct pattern for passing to non-ECS helpers

## Messages::drain() as alternative to MessageReader in direct World access

The orchestration system uses `world.resource_mut::<Messages<T>>().drain()` instead of
`MessageReader<T>`. This is a valid alternative when running as a `&mut World` system that
cannot receive `SystemParam` arguments. `drain()` consumes all messages from both internal
buffers — appropriate for the orchestrator which is the sole consumer of these internal
lifecycle messages.

## fire_dispatch() inside exclusive systems (Bevy 0.18)

`fire_dispatch(&EffectType::Foo(config), entity, source, world)` is project-defined (not a
Bevy built-in). Its signature is `fn fire_dispatch(effect: &EffectType, entity: Entity, source: &str, world: &mut World)`.
It is correct and idiomatic to call it multiple times sequentially inside an exclusive system
(`pub fn tick_circuit_breaker(world: &mut World)`). Each call completes and drops its internal
borrows before the next call begins — no aliasing issue.

## &Newtype(inner) destructure in Query for-loop (Bevy 0.18, Copy inner type)

Pattern: `for (..., &MyNewtype(val), ...) in &query` where `MyNewtype(pub f32)` has no
`#[derive(Copy)]`. This compiles because:
- `&query` yields `&MyNewtype` for that position
- The pattern `&MyNewtype(val)` match-dereferences the shared reference  
- `f32: Copy` so Rust binds `val` as `f32` by copy (match ergonomics + Copy)
- This does NOT require `MyNewtype` to implement `Copy` — only the inner field must be `Copy`
Confirmed correct in `tick_tether_beam` for `&TetherBeamWidth(beam_width)` where `beam_width: f32`.

## world.resource_mut::<T>() in exclusive system (Bevy 0.18)

Exclusive systems (`fn my_sys(world: &mut World)`) can call `world.resource_mut::<T>()` freely.
The returned `Mut<T>` borrow is bounded to a local scope — once it drops, `world` is unlocked
again. Multiple sequential calls to `world.resource_mut::<GameRng>()` (or any resource) inside
one exclusive system are safe as long as they don't overlap. Pattern confirmed correct in
`SpawnBoltsConfig::fire()` and `TetherBeamConfig::fire_spawn()`.

## derive_partial_eq_without_eq lint — only triggers when PartialEq IS derived

The project has `derive_partial_eq_without_eq = "deny"`. This lint fires ONLY when `PartialEq`
is derived without `Eq`. Components with `#[derive(Component, Debug, Clone)]` (no `PartialEq`)
are completely exempt — the lint does not apply.

## chip.and_then(|c| c.0.clone()) — Option<&EffectSourceChip> → Option<String>

`EffectSourceChip(pub Option<String>)`. Pattern:
```rust
let source_chip: Option<String> = chip.and_then(|c| c.0.clone());
```
where `chip: Option<&EffectSourceChip>`. The closure receives `&EffectSourceChip`, accesses
`.0` (the `Option<String>`), clones it to get `Option<String>`, and `and_then` flattens from
`Option<Option<String>>` to `Option<String>`. Correct and idiomatic in Bevy 0.18.
