//! Handles keyboard text input for the seed entry field.

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use crate::state::menu::start_game::resources::SeedEntry;

/// Maximum characters in the seed entry (u64 max is 20 digits).
const MAX_SEED_CHARS: usize = 18;

/// Reads `KeyboardInput` messages to update the [`SeedEntry`] resource.
///
/// - Tab toggles focus on/off
/// - When focused: digit keys append, Backspace deletes, Escape unfocuses
/// - When unfocused: returns early (no input consumed)
pub(crate) fn handle_seed_input(
    mut reader: MessageReader<KeyboardInput>,
    mut seed_entry: ResMut<SeedEntry>,
) {
    for input in reader.read() {
        if !input.state.is_pressed() {
            continue;
        }

        // Tab toggles focus regardless of current state
        if input.logical_key == Key::Tab {
            seed_entry.focused = !seed_entry.focused;
            continue;
        }

        if !seed_entry.focused {
            continue;
        }

        match &input.logical_key {
            Key::Backspace => {
                seed_entry.value.pop();
            }
            Key::Escape => {
                seed_entry.focused = false;
            }
            _ => {
                if let Some(ref text) = input.text {
                    for c in text.chars() {
                        if c.is_ascii_digit() && seed_entry.value.len() < MAX_SEED_CHARS {
                            seed_entry.value.push(c);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::{ButtonState, keyboard::Key};

    use super::*;

    fn test_app() -> App {
        use crate::prelude::*;
        TestAppBuilder::new()
            .with_message::<KeyboardInput>()
            .with_resource::<SeedEntry>()
            .with_system(Update, handle_seed_input)
            .build()
    }

    fn send_key(app: &mut App, logical_key: Key, text: Option<&str>) {
        let entity = app.world_mut().spawn_empty().id();
        let msg = KeyboardInput {
            key_code: KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
            logical_key,
            state: ButtonState::Pressed,
            text: text.map(Into::into),
            repeat: false,
            window: entity,
        };
        app.world_mut().write_message(msg);
        app.update();
    }

    #[test]
    fn tab_toggles_focus() {
        let mut app = test_app();
        assert!(!app.world().resource::<SeedEntry>().focused);

        send_key(&mut app, Key::Tab, None);
        assert!(app.world().resource::<SeedEntry>().focused);

        send_key(&mut app, Key::Tab, None);
        assert!(!app.world().resource::<SeedEntry>().focused);
    }

    #[test]
    fn digit_input_when_focused() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SeedEntry>().focused = true;

        send_key(&mut app, Key::Character("1".into()), Some("1"));
        send_key(&mut app, Key::Character("2".into()), Some("2"));
        send_key(&mut app, Key::Character("3".into()), Some("3"));

        assert_eq!(app.world().resource::<SeedEntry>().value, "123");
    }

    #[test]
    fn digit_input_ignored_when_unfocused() {
        let mut app = test_app();
        // Not focused
        send_key(&mut app, Key::Character("5".into()), Some("5"));

        assert!(app.world().resource::<SeedEntry>().value.is_empty());
    }

    #[test]
    fn non_digit_input_ignored() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SeedEntry>().focused = true;

        send_key(&mut app, Key::Character("a".into()), Some("a"));
        send_key(&mut app, Key::Character("!".into()), Some("!"));

        assert!(app.world().resource::<SeedEntry>().value.is_empty());
    }

    #[test]
    fn backspace_removes_last_char() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SeedEntry>().focused = true;
        app.world_mut().resource_mut::<SeedEntry>().value = "123".to_owned();

        send_key(&mut app, Key::Backspace, None);

        assert_eq!(app.world().resource::<SeedEntry>().value, "12");
    }

    #[test]
    fn escape_unfocuses() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SeedEntry>().focused = true;

        send_key(&mut app, Key::Escape, None);

        assert!(!app.world().resource::<SeedEntry>().focused);
    }

    #[test]
    fn max_chars_enforced() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SeedEntry>().focused = true;
        app.world_mut().resource_mut::<SeedEntry>().value = "1".repeat(MAX_SEED_CHARS);

        send_key(&mut app, Key::Character("9".into()), Some("9"));

        assert_eq!(
            app.world().resource::<SeedEntry>().value.len(),
            MAX_SEED_CHARS
        );
    }

    #[test]
    fn release_events_ignored() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SeedEntry>().focused = true;

        let entity = app.world_mut().spawn_empty().id();
        let msg = KeyboardInput {
            key_code:    KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
            logical_key: Key::Character("5".into()),
            state:       ButtonState::Released,
            text:        Some("5".into()),
            repeat:      false,
            window:      entity,
        };
        app.world_mut().write_message(msg);
        app.update();

        assert!(app.world().resource::<SeedEntry>().value.is_empty());
    }
}
