---
name: Bevy 0.18.1 World::commands() and World::flush()
description: World::commands() returns Commands backed by world's internal queue; World::flush() applies that queue — verified against docs.rs/bevy/0.18.1 and Bevy source
type: reference
---

# World::commands() and World::flush() — Bevy 0.18.1

Verified against: docs.rs/bevy/0.18.1, github.com/bevyengine/bevy/tree/v0.18.1

## World::commands()

```rust
pub fn commands(&mut self) -> Commands<'_, '_>
```

Doc: "Creates a new `Commands` instance that writes to the world's command queue. Use `World::flush` to apply all queued commands."

- Takes `&mut self`
- Returns `Commands<'_, '_>` — two lifetime parameters (world and queue)
- The returned `Commands` writes to the **world's own internal command queue** (not a separate `CommandQueue`)
- All builder methods (`spawn(bundle)`, `entity(id).insert(...)`, etc.) work normally
- Commands are NOT applied until `World::flush()` is called

Implementation (from source):
```rust
#[inline]
pub fn commands(&mut self) -> Commands<'_, '_> {
    // SAFETY: command_queue is stored on world and always valid while the world exists
    unsafe {
        Commands::new_raw_from_entities(
            self.command_queue.clone(),
            &self.allocator,
            &self.entities,
        )
    }
}
```

## World::flush()

```rust
pub fn flush(&mut self)
```

- Takes only `&mut self`, no parameters, returns `()`
- Applies all commands queued in the world's internal command queue
- Also called automatically in several World methods (spawn, modify_component, etc.)
- There is NO `flush_commands()` — the method is just `flush()`

## World::entities_and_commands()

```rust
pub fn entities_and_commands(&mut self) -> (EntityFetcher<'_>, Commands<'_, '_>)
```

Provides simultaneous access to entity fetching and command queuing.

## Idiomatic test pattern in Bevy 0.18.1

```rust
// Clean — no manual CommandQueue needed
fn spawn_in_world(world: &mut World, bundle: impl Bundle) -> Entity {
    let entity = world.commands().spawn(bundle).id();
    world.flush();
    entity
}

// Or inline:
let entity = world.commands().spawn((MyComponent, OtherComponent)).id();
world.flush();
assert!(world.get::<MyComponent>(entity).is_some());
```

## Comparison with manual CommandQueue pattern

The manual pattern used in Bevy's own `commands/mod.rs` tests:
```rust
let mut command_queue = CommandQueue::default();
Commands::new(&mut command_queue, &world).spawn(bundle);
command_queue.apply(&mut world);
```

This is equivalent but more verbose. `World::commands()` + `World::flush()` is the simpler alternative that avoids manually managing a `CommandQueue`.

Note: Bevy's own internal tests (in `commands/mod.rs`) still use the manual pattern as of 0.18.1, but the API exists and is documented for user code.

## Gotchas

- `World::flush()` applies ALL pending commands, including any queued by internal world operations. This is fine for tests but be aware in production code.
- The returned `Commands<'_, '_>` borrows `&mut World` — you cannot use the world for anything else while the `Commands` borrow is live. Drop or let the `Commands` go out of scope before calling `world.flush()` or doing other world operations.
- `flush_commands()` does NOT exist — the method is `flush()`.
- This API predates 0.18 — no migration note in 0.17→0.18 guide, meaning it's been stable across versions.
