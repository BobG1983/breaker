---
name: KeyboardInput API (Bevy 0.18.1)
description: Verified KeyboardInput struct fields, message system usage, InputSystems set, and test patterns for keyboard input in Bevy 0.18.1
type: reference
---

# KeyboardInput — Bevy 0.18.1 (verified from bevy_input-0.18.1 source)

## KeyboardInput is a Message, NOT an Event

`KeyboardInput` derives `#[derive(Message)]` and is registered via `add_message::<KeyboardInput>()`.
It is NOT an `Event`. Do NOT use `EventReader<KeyboardInput>` — that will not compile.

Use `MessageReader<KeyboardInput>` in systems.

## KeyboardInput Struct (all fields)

```rust
// bevy_input::keyboard::KeyboardInput
#[derive(Message, Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardInput {
    pub key_code: KeyCode,        // Physical key code (layout-independent)
    pub logical_key: Key,         // Logical key (layout-aware)
    pub state: ButtonState,       // Pressed or Released
    pub text: Option<SmolStr>,    // Text produced by this keypress (None if not text-producing)
    pub repeat: bool,             // True if this is a key-repeat event
    pub window: Entity,           // Window entity that received the input
}
```

`SmolStr` is from the `smol_str` crate (or `alloc::string::String` on no_std). The `text` field IS
present in 0.18.1 — if you're getting "missing field `text`" it means you're constructing
`KeyboardInput` with struct literal syntax and omitting it. You must include `text: None` (or the
appropriate value).

## ButtonState

```rust
pub enum ButtonState {
    Pressed,
    Released,
}
impl ButtonState {
    pub fn is_pressed(&self) -> bool { ... }
}
```

## InputSystems Set (PreUpdate schedule)

The system set is called `InputSystems` (plural), NOT `InputSystem`.

```rust
// bevy_input::InputSystems
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct InputSystems;
```

Usage: `.after(InputSystems)` or `.in_set(InputSystems)`.
Registered in `PreUpdate` schedule by `InputPlugin`.

## Reading KeyboardInput in a System

```rust
use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;

fn my_system(mut reader: MessageReader<KeyboardInput>) {
    for event in reader.read() {
        if event.key_code == KeyCode::Space && event.state.is_pressed() && !event.repeat {
            // handle space press
        }
    }
}
```

## Sending KeyboardInput in Tests

`app.world_mut().send_event(...)` does NOT exist in Bevy 0.18.1 — `KeyboardInput` is a Message,
not an Event. There are two correct approaches:

### Approach 1: world_mut().write_message() (direct, one-shot)

```rust
// Requires a window entity. For tests without a real window, spawn a dummy entity.
let window = app.world_mut().spawn_empty().id();
app.world_mut().write_message(KeyboardInput {
    key_code: KeyCode::ArrowLeft,
    logical_key: Key::ArrowLeft,
    state: ButtonState::Pressed,
    text: None,
    repeat: false,
    window,
});
app.update();
```

`World::write_message<M: Message>` returns `Option<MessageId<M>>` — it returns `None` if the
`Messages<M>` resource has not been added.

### Approach 2: Enqueue helper system (project's established pattern)

This is what the project already uses (see `src/cells/systems/handle_cell_hit.rs`):

```rust
#[derive(Resource)]
struct TestKeyInput(Option<KeyboardInput>);

fn enqueue_key_input(res: Res<TestKeyInput>, mut writer: MessageWriter<KeyboardInput>) {
    if let Some(input) = res.0.clone() {
        writer.write(input);
    }
}

// In test_app():
app.insert_resource(TestKeyInput(None));
app.add_systems(Update, enqueue_key_input.before(my_system_under_test));

// In test body:
*app.world_mut().resource_mut::<TestKeyInput>() = TestKeyInput(Some(KeyboardInput {
    key_code: KeyCode::ArrowLeft,
    logical_key: Key::ArrowLeft,
    state: ButtonState::Pressed,
    text: None,
    repeat: false,
    window: Entity::PLACEHOLDER,  // or a real entity
}));
app.update();
```

## Import paths

```rust
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::input::{ButtonInput, ButtonState, InputSystems};
// Or via prelude:
use bevy::prelude::*;  // KeyCode, ButtonInput in prelude; KeyboardInput, Key, ButtonState need explicit import
```

`KeyCode` and `ButtonInput` are in `bevy::prelude`. `KeyboardInput`, `Key`, and `ButtonState`
are NOT in `bevy::prelude` — use `bevy::input::keyboard::KeyboardInput` etc.

## Also registered as Messages (same plugin)

- `KeyboardFocusLost` — fired when the window loses focus, clears stuck key states
- `ButtonInput<KeyCode>` and `ButtonInput<Key>` resources updated in `InputSystems`
