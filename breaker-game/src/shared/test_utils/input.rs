//! Test helpers for driving keyboard input through the real input pipeline.
//!
//! Tests must NOT mutate `ButtonInput<KeyCode>` directly via `press` — the
//! `InputPlugin`'s `PreUpdate` system clears `just_pressed` at frame start,
//! so a direct mutation is wiped out before the `Update`-schedule system
//! under test runs.
//!
//! These helpers write `KeyboardInput` messages so `InputPlugin`'s
//! `keyboard_input_system` processes them during `PreUpdate` and sets
//! `just_pressed` / `just_released` correctly for the `Update`-schedule
//! reader.

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput, NativeKey},
    },
    prelude::*,
};

fn write_keyboard_event(app: &mut App, key_code: KeyCode, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        window: Entity::PLACEHOLDER,
        repeat: false,
    });
}

/// Writes a press AND release event for `key_code` in the same frame, then
/// runs one `app.update()`. Tap semantics: the system sees `just_pressed`
/// during this frame, and the key is left in the not-pressed state so a
/// subsequent `press_key` for the same key fires `just_pressed` again.
pub(crate) fn press_key(app: &mut App, key_code: KeyCode) {
    write_keyboard_event(app, key_code, ButtonState::Pressed);
    write_keyboard_event(app, key_code, ButtonState::Released);
    app.update();
}
