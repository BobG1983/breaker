---
name: Bevy 0.18.1 Message System
description: Message derive, MessageWriter/Reader, registration, test injection, on_message run condition, AppExit
type: reference
---

# Message System (Bevy 0.18.1)

Verified against docs.rs/bevy_ecs/0.18.1, docs.rs/bevy_ecs_macros/0.18.1, github.com/bevyengine/bevy/tree/v0.18.1.

## `#[derive(Message)]` on generic structs — CONFIRMED WORKS

The `derive_message` macro in `bevy_ecs_macros/src/message.rs`:
1. Calls `ast.generics.split_for_impl()` — preserves all user generics
2. Appends `Self: Send + Sync + 'static` to the where clause
3. Emits an empty impl block: `impl<T> Message for Foo<T> where Self: Send + Sync + 'static {}`

**Proof**: `StateTransitionEvent<S: States>` in bevy_state uses `#[derive(..., Message)]` on a
generic struct — verified from bevy_state source.

`Message` trait bounds: `Send + Sync + 'static` only.
`MessageReader<E>`: bound `E: Message` only.
`MessageWriter<E>`: bound `E: Message` only.

`Messages<ChangeState<NodeState>>` and `Messages<ChangeState<RunState>>` are distinct resources
(different `TypeId`). Each instantiation needs its own `app.add_message::<T>()` call.

## Core types

| Type | Role |
|------|------|
| `Messages<T>` | `Resource` — the actual message storage (double-buffered) |
| `MessageWriter<T>` | `SystemParam` — thin wrapper around `ResMut<Messages<T>>` |
| `MessageReader<T>` | `SystemParam` — reads from `Messages<T>` with cursor tracking |
| `MessageMutator<T>` | `SystemParam` — mutable read with cursor tracking |
| `MessageCursor<T>` | Tracks per-reader position in the message buffer |

## Registering a message type

```rust
app.add_message::<MyMessage>();
// Must be called before any system reads/writes the message.
// Inserts Messages<MyMessage> as a resource and schedules update system.
```

## Writing messages from a system

```rust
fn my_system(mut writer: MessageWriter<MyMessage>) {
    writer.write(MyMessage { ... });
}
```

## Writing messages directly from &mut World (in tests)

```rust
// Option 1 — resource_mut (most explicit, recommended for tests):
app.world_mut()
    .resource_mut::<Messages<MyMessage>>()
    .write(MyMessage { ... });

// Option 2 — World::write_message convenience method:
app.world_mut().write_message(MyMessage { ... });

// Batch variant:
app.world_mut()
    .resource_mut::<Messages<MyMessage>>()
    .write_batch([msg1, msg2]);
```

`World::write_message` / `write_message_batch` / `write_message_default` are confirmed
in the World method list on docs.rs. Logs an error if the type was not registered.

## Reading messages from a system

```rust
fn my_system(mut reader: MessageReader<MyMessage>) {
    for msg in reader.read() {
        // msg: &MyMessage
    }
}
```

## Message update / buffer swap

`Messages::update()` swaps double buffers once per frame. This is handled automatically by
`message_update_system` (scheduled in `app.add_message()`). Tests using `app.update()` or
`tick()` will have messages visible to readers on the same update tick they are written.

## Module path

```rust
use bevy::ecs::message::{Messages, MessageWriter, MessageReader, MessageCursor};
// All re-exported in bevy::prelude
use bevy::prelude::*;
```

---

## `on_message` run condition (Bevy 0.18.1)

Verified from source: `github.com/bevyengine/bevy/blob/v0.18.1/crates/bevy_ecs/src/schedule/condition.rs`

```rust
// Exact signature:
pub fn on_message<M: Message>(reader: MessageReader<'_, '_, M>) -> bool

// Module path:
use bevy::ecs::schedule::common_conditions::on_message;
// Also re-exported in bevy::prelude::*

// Usage:
.add_systems(Update, my_system.run_if(on_message::<MyMessage>()))
```

**CRITICAL**: `on_message` uses its own `MessageReader` with its own `Local<MessageCursor<M>>`.
Each `MessageReader` instance (whether in a run condition or system body) has an **independent cursor**.
The condition advancing its cursor does NOT consume messages from the system body's reader.

The condition returns `true` when new messages exist AND advances the condition's own cursor to prevent
re-firing on the same message batch. The system body's `MessageReader` still sees all messages.

---

## AppExit — Message-Based App Shutdown

### AppExit is a Message, not an Event

```rust
#[derive(Message, Debug, Clone, Default, PartialEq, Eq)]
pub enum AppExit { #[default] Success, Error(NonZero<u8>) }
```

### Registration — automatic, never call add_message::<AppExit>()

`App::default()` always calls `app.add_message::<AppExit>()` (line 131, app.rs). It is always
available as a `MessageWriter<AppExit>` system param. Never register it yourself.

### Writing AppExit

```rust
fn quit(mut writer: MessageWriter<AppExit>) {
    writer.write(AppExit::Success);  // app exits after this frame
}
```

### When does the runner check for AppExit?

**Not inside the schedule.** The check is in the runner, outside ECS, after `app.update()` returns.
`App::should_exit()` creates a fresh `MessageCursor` (last_message_count=0) and reads `Messages<AppExit>`.
Because the cursor starts at 0, it reads the FULL double-buffer — so the message is visible
even after `message_update_system` has swapped buffers.

- **Windowed apps (DefaultPlugins/winit)**: checked in `redraw_requested()` after each `app.update()` call
- **Headless apps (ScheduleRunnerPlugin)**: checked after each loop iteration

### Hang-on-quit: the most likely cause

`UpdateMode::Reactive` — winit only calls `app.update()` when events arrive. If no events fire
after `AppExit` is written, `should_exit()` is never called and the message sits unchecked.
Solution: ensure an input event fires, OR trigger a `RequestRedraw` message alongside `AppExit`.

### Error priority

If both `AppExit::Success` and `AppExit::Error(N)` exist in the buffer, `should_exit()` returns
the first `Error` it finds.
