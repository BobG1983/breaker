//! Core input translation system.

use bevy::{input::keyboard::KeyboardInput, prelude::*};

use crate::input::resources::{GameAction, InputActions, InputConfig};

/// Tracks timestamps for double-tap dash detection.
#[derive(Resource, Debug)]
pub struct DoubleTapState {
    /// Wall-clock time of the last left-direction press.
    pub last_left_tap: f64,
    /// Wall-clock time of the last right-direction press.
    pub last_right_tap: f64,
}

impl Default for DoubleTapState {
    fn default() -> Self {
        Self {
            last_left_tap: f64::NEG_INFINITY,
            last_right_tap: f64::NEG_INFINITY,
        }
    }
}

/// Translates raw keyboard input into [`InputActions`].
///
/// Runs in `PreUpdate` after `InputSystems`. Reads held keys for movement
/// and `MessageReader<KeyboardInput>` for one-shot presses (bump, dash).
pub fn read_input_actions(
    keyboard: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    time: Res<Time<Real>>,
    mut actions: ResMut<InputActions>,
    mut double_tap: ResMut<DoubleTapState>,
    mut key_events: MessageReader<KeyboardInput>,
) {
    actions.0.clear();

    // Held keys → continuous movement
    if config.move_left.iter().any(|k| keyboard.pressed(*k)) {
        actions.0.push(GameAction::MoveLeft);
    }
    if config.move_right.iter().any(|k| keyboard.pressed(*k)) {
        actions.0.push(GameAction::MoveRight);
    }

    // One-shot key presses via messages (FixedUpdate-safe, double-buffered)
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
                double_tap.last_left_tap = 0.0; // consume
            } else {
                double_tap.last_left_tap = now;
            }
        }

        // Double-tap right
        if config.move_right.contains(&key) {
            if now - double_tap.last_right_tap < f64::from(config.double_tap_window) {
                if !actions.active(GameAction::DashRight) {
                    actions.0.push(GameAction::DashRight);
                }
                double_tap.last_right_tap = 0.0; // consume
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
        app.add_plugins(MinimalPlugins);
        app.init_resource::<InputActions>();
        app.init_resource::<InputConfig>();
        app.init_resource::<DoubleTapState>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_message::<KeyboardInput>();
        app.add_systems(Update, read_input_actions);
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
    fn slow_double_tap_does_not_dash() {
        let mut app = test_app();
        let config = app.world().resource::<InputConfig>().clone();

        // First tap
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();

        // Advance real time past the double-tap window
        let wait = std::time::Duration::from_secs_f64(f64::from(config.double_tap_window) + 0.5);
        std::thread::sleep(wait);

        // Second tap
        send_key_press(&mut app, KeyCode::ArrowLeft);
        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(
            !actions.active(GameAction::DashLeft),
            "slow double-tap should not trigger dash"
        );
    }
}
