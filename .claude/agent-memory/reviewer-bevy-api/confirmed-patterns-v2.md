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

## Messages::drain() as alternative to MessageReader in direct World access

The orchestration system uses `world.resource_mut::<Messages<T>>().drain()` instead of
`MessageReader<T>`. This is a valid alternative when running as a `&mut World` system that
cannot receive `SystemParam` arguments. `drain()` consumes all messages from both internal
buffers — appropriate for the orchestrator which is the sole consumer of these internal
lifecycle messages.
