//! Core input translation system.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::input::resources::*;

/// Clears [`InputActions`] at the end of each fixed tick.
///
/// Runs in `FixedPostUpdate` so actions injected in `FixedPreUpdate` or
/// `PreUpdate` persist through `FixedUpdate` before being cleared.
pub fn clear_input_actions(mut actions: ResMut<InputActions>) {
    actions.0.clear();
}

/// Translates raw keyboard input into [`InputActions`].
///
/// Runs in `PreUpdate` after `InputSystems`. Reads held keys for movement
/// and `MessageReader<KeyboardInput>` for one-shot presses (bump, dash).
/// Does not clear — [`clear_input_actions`] handles that in `FixedPostUpdate`.
pub fn read_input_actions(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    time: Res<Time<Real>>,
    mut actions: ResMut<InputActions>,
    mut double_tap: ResMut<DoubleTapState>,
    mut key_events: MessageReader<KeyboardInput>,
) {
    // Held keys → continuous movement
    if config.move_left.iter().any(|k| keyboard.pressed(*k)) {
        actions.0.push(GameAction::MoveLeft);
    }
    if config.move_right.iter().any(|k| keyboard.pressed(*k)) {
        actions.0.push(GameAction::MoveRight);
    }

    // One-shot key presses via messages (FixedUpdate-safe, double-buffered).
    // Wall-clock time (Time<Real>) for double-tap detection — players expect the
    // tap window to track real time, not simulation time.
    let now = time.elapsed_secs_f64();

    for event in key_events.read() {
        if !event.state.is_pressed() || event.repeat {
            continue;
        }

        let key = event.key_code;

        // Bump
        if config.bump.contains(&key) && !actions.active(GameAction::Bump) {
            actions.0.push(GameAction::Bump);
        }

        // Double-tap left
        if config.move_left.contains(&key) {
            if now - double_tap.last_left_tap < f64::from(config.double_tap_window) {
                if !actions.active(GameAction::DashLeft) {
                    actions.0.push(GameAction::DashLeft);
                }
                double_tap.last_left_tap = f64::NEG_INFINITY; // consume
            } else {
                double_tap.last_left_tap = now;
            }
        }

        // Pause toggle
        if key == KeyCode::Escape && !actions.active(GameAction::TogglePause) {
            actions.0.push(GameAction::TogglePause);
        }

        // Menu actions
        if config.menu_up.contains(&key) && !actions.active(GameAction::MenuUp) {
            actions.0.push(GameAction::MenuUp);
        }
        if config.menu_down.contains(&key) && !actions.active(GameAction::MenuDown) {
            actions.0.push(GameAction::MenuDown);
        }
        if config.menu_left.contains(&key) && !actions.active(GameAction::MenuLeft) {
            actions.0.push(GameAction::MenuLeft);
        }
        if config.menu_right.contains(&key) && !actions.active(GameAction::MenuRight) {
            actions.0.push(GameAction::MenuRight);
        }
        if config.menu_confirm.contains(&key) && !actions.active(GameAction::MenuConfirm) {
            actions.0.push(GameAction::MenuConfirm);
        }

        // Double-tap right
        if config.move_right.contains(&key) {
            if now - double_tap.last_right_tap < f64::from(config.double_tap_window) {
                if !actions.active(GameAction::DashRight) {
                    actions.0.push(GameAction::DashRight);
                }
                double_tap.last_right_tap = f64::NEG_INFINITY; // consume
            } else {
                double_tap.last_right_tap = now;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::{ButtonState, keyboard::KeyboardInput};

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InputActions>()
            .init_resource::<InputConfig>()
            .init_resource::<DoubleTapState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<KeyboardInput>()
            .add_systems(Update, read_input_actions);
        app
    }

    fn send_key_press(app: &mut App, key: KeyCode) {
        app.world_mut().write_message(KeyboardInput {
            key_code: key,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state: ButtonState::Pressed,
            text: None,
            window: Entity::PLACEHOLDER,
            repeat: false,
        });
    }

    fn send_key_release(app: &mut App, key: KeyCode) {
        app.world_mut().write_message(KeyboardInput {
            key_code: key,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state: ButtonState::Released,
            text: None,
            window: Entity::PLACEHOLDER,
            repeat: false,
        });
    }

    #[test]
    fn no_actions_without_input() {
        let mut app = test_app();
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.0.is_empty());
    }

    #[test]
    fn move_left_on_bound_key_held() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::MoveLeft));
    }

    #[test]
    fn move_right_on_bound_key_held() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowRight);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::MoveRight));
    }

    #[test]
    fn bump_action_on_bound_key_press() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::ArrowUp);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::Bump));
    }

    #[test]
    fn release_events_ignored() {
        let mut app = test_app();
        send_key_release(&mut app, KeyCode::ArrowUp);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(!actions.active(GameAction::Bump));
    }

    #[test]
    fn repeat_key_events_ignored() {
        let mut app = test_app();
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::ArrowUp,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            state: ButtonState::Pressed,
            text: None,
            window: Entity::PLACEHOLDER,
            repeat: true,
        });
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(
            !actions.active(GameAction::Bump),
            "repeat key press should not produce Bump action"
        );
    }

    #[test]
    fn double_tap_left_triggers_dash_left() {
        let mut app = test_app();

        // First tap — records timestamp
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(
            !actions.active(GameAction::DashLeft),
            "first tap should not dash"
        );

        // Second tap — within window
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(
            actions.active(GameAction::DashLeft),
            "second tap should trigger dash left"
        );
    }

    #[test]
    fn double_tap_right_triggers_dash_right() {
        let mut app = test_app();

        send_key_press(&mut app, KeyCode::ArrowRight);
        app.update();

        send_key_press(&mut app, KeyCode::ArrowRight);
        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::DashRight));
    }

    #[test]
    fn single_tap_does_not_dash() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(!actions.active(GameAction::DashLeft));
        assert!(!actions.active(GameAction::DashRight));
    }

    #[test]
    fn menu_up_key_produces_menu_up_action() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::ArrowUp);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::MenuUp));
    }

    #[test]
    fn menu_down_key_produces_menu_down_action() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::ArrowDown);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::MenuDown));
    }

    #[test]
    fn menu_left_key_produces_menu_left_action() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::MenuLeft));
    }

    #[test]
    fn menu_right_key_produces_menu_right_action() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::ArrowRight);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::MenuRight));
    }

    #[test]
    fn menu_confirm_key_produces_menu_confirm_action() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::Enter);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(actions.active(GameAction::MenuConfirm));
    }

    #[test]
    fn escape_key_produces_toggle_pause_action() {
        let mut app = test_app();
        send_key_press(&mut app, KeyCode::Escape);
        app.update();
        let actions = app.world().resource::<InputActions>();
        assert!(
            actions.active(GameAction::TogglePause),
            "Escape key should produce GameAction::TogglePause"
        );
    }

    #[test]
    fn slow_double_tap_does_not_dash() {
        use bevy::time::TimeUpdateStrategy;

        let mut app = test_app();
        let config = app.world().resource::<InputConfig>().clone();

        // Use manual time advancement instead of thread::sleep
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(16),
        ));

        // First tap — records timestamp
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();

        // Advance time past the double-tap window
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(f64::from(config.double_tap_window) + 0.5),
        ));

        // Second tap — outside window
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(
            !actions.active(GameAction::DashLeft),
            "slow double-tap should not trigger dash"
        );
    }
}
