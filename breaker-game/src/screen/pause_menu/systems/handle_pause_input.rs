//! Handles keyboard input on the pause menu.

use bevy::prelude::*;

use crate::{
    input::InputConfig,
    screen::pause_menu::{
        components::{PAUSE_MENU_ITEMS, PauseMenuItem},
        resources::PauseMenuSelection,
    },
    shared::{GameState, PlayingState},
};

/// Handles keyboard navigation and confirmation on the pause menu.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as main menu).
pub(crate) fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    mut selection: ResMut<PauseMenuSelection>,
    mut next_playing_state: ResMut<NextState<PlayingState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    // Navigate down
    if config.menu_down.iter().any(|k| keys.just_pressed(*k)) {
        let current = current_index(&selection);
        let next = (current + 1) % PAUSE_MENU_ITEMS.len();
        selection.selected = PAUSE_MENU_ITEMS[next];
    }

    // Navigate up
    if config.menu_up.iter().any(|k| keys.just_pressed(*k)) {
        let current = current_index(&selection);
        let next = if current == 0 {
            PAUSE_MENU_ITEMS.len() - 1
        } else {
            current - 1
        };
        selection.selected = PAUSE_MENU_ITEMS[next];
    }

    // Confirm
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        match selection.selected {
            PauseMenuItem::Resume => {
                next_playing_state.set(PlayingState::Active);
            }
            PauseMenuItem::Quit => {
                next_game_state.set(GameState::MainMenu);
            }
        }
    }
}

fn current_index(selection: &PauseMenuSelection) -> usize {
    PAUSE_MENU_ITEMS
        .iter()
        .position(|item| *item == selection.selected)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(InputConfig::default())
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            .insert_resource(PauseMenuSelection {
                selected: PauseMenuItem::Resume,
            })
            .add_systems(Update, handle_pause_input);
        app
    }

    fn press_key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
    }

    #[test]
    fn down_press_advances_to_quit() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowDown);

        let selection = app.world().resource::<PauseMenuSelection>();
        assert_eq!(selection.selected, PauseMenuItem::Quit);
    }

    #[test]
    fn up_press_wraps_to_quit() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowUp);

        let selection = app.world().resource::<PauseMenuSelection>();
        assert_eq!(selection.selected, PauseMenuItem::Quit);
    }

    #[test]
    fn confirm_resume_sets_playing_active() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<PlayingState>>();
        assert!(
            format!("{next:?}").contains("Active"),
            "expected Active, got: {next:?}"
        );
    }

    #[test]
    fn confirm_quit_transitions_to_main_menu() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<PauseMenuSelection>()
            .selected = PauseMenuItem::Quit;
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("MainMenu"),
            "expected MainMenu, got: {next:?}"
        );
    }

    #[test]
    fn navigation_wraps_down() {
        let mut app = test_app();
        // Down twice wraps back to Resume
        press_key(&mut app, KeyCode::ArrowDown);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowDown);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();

        press_key(&mut app, KeyCode::ArrowDown);

        let selection = app.world().resource::<PauseMenuSelection>();
        assert_eq!(selection.selected, PauseMenuItem::Resume);
    }
}
