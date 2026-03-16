//! Handles keyboard input on the upgrade selection screen.

use bevy::prelude::*;

use crate::{
    input::InputConfig,
    screen::upgrade_select::{components::CARD_COUNT, resources::UpgradeSelectSelection},
    shared::GameState,
};

/// Handles left/right card navigation and confirmation.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as other menus).
pub fn handle_upgrade_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    mut selection: ResMut<UpgradeSelectSelection>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Horizontal card layout reuses move_left/move_right; no dedicated
    // menu_left/menu_right binding exists.
    // Navigate left
    if config.move_left.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = if selection.index == 0 {
            CARD_COUNT - 1
        } else {
            selection.index - 1
        };
    }

    // Navigate right
    if config.move_right.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = (selection.index + 1) % CARD_COUNT;
    }

    // Confirm selection
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        next_state.set(GameState::NodeTransition);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.insert_resource(InputConfig::default());
        app.init_state::<GameState>();
        app.insert_resource(UpgradeSelectSelection { index: 0 });
        app.add_systems(Update, handle_upgrade_input);
        app
    }

    fn press_key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
    }

    #[test]
    fn right_advances_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowRight);

        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 1);
    }

    #[test]
    fn left_wraps_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowLeft);

        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 2); // wraps from 0 to last (2)
    }

    #[test]
    fn confirm_transitions_to_node_transition() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("NodeTransition"),
            "expected NodeTransition, got: {next:?}"
        );
    }

    #[test]
    fn right_wraps_around() {
        let mut app = test_app();
        // Go right 3 times to wrap around
        for _ in 0..3 {
            press_key(&mut app, KeyCode::ArrowRight);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::ArrowRight);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        }

        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 0); // wraps back to 0
    }

    #[test]
    fn no_input_no_change() {
        let mut app = test_app();
        app.update();

        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 0);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            !format!("{next:?}").contains("NodeTransition"),
            "expected no transition, got: {next:?}"
        );
    }
}
