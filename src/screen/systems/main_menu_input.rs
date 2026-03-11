//! Main menu keyboard and mouse input handling.

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::shared::GameState;

use crate::screen::components::{MENU_ITEMS, MenuItem};
use crate::screen::resources::MainMenuSelection;

/// Handles keyboard and mouse input for the main menu.
pub fn handle_main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<MainMenuSelection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit_writer: MessageWriter<AppExit>,
    interaction_query: Query<(&Interaction, &MenuItem), Changed<Interaction>>,
) {
    // Mouse interaction
    for (interaction, item) in &interaction_query {
        match interaction {
            Interaction::Pressed => {
                selection.selected = *item;
                confirm_selection(&selection, &mut next_state, &mut exit_writer);
            }
            Interaction::Hovered => {
                selection.selected = *item;
            }
            Interaction::None => {}
        }
    }

    // Keyboard navigation
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        let current = current_index(&selection);
        let next = (current + 1) % MENU_ITEMS.len();
        selection.selected = MENU_ITEMS[next];
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        let current = current_index(&selection);
        let next = if current == 0 {
            MENU_ITEMS.len() - 1
        } else {
            current - 1
        };
        selection.selected = MENU_ITEMS[next];
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        confirm_selection(&selection, &mut next_state, &mut exit_writer);
    }
}

/// Returns the index of the currently selected item in [`MENU_ITEMS`].
fn current_index(selection: &MainMenuSelection) -> usize {
    MENU_ITEMS
        .iter()
        .position(|item| *item == selection.selected)
        .unwrap_or(0)
}

/// Executes the action for the current selection.
fn confirm_selection(
    selection: &MainMenuSelection,
    next_state: &mut ResMut<NextState<GameState>>,
    exit_writer: &mut MessageWriter<AppExit>,
) {
    match selection.selected {
        MenuItem::Play => next_state.set(GameState::Playing),
        MenuItem::Settings => {} // Not yet implemented
        MenuItem::Quit => {
            exit_writer.write(AppExit::Success);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::message::Messages;
    use bevy::state::app::StatesPlugin;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_state::<GameState>();
        app.add_message::<AppExit>();
        app.insert_resource(MainMenuSelection {
            selected: MenuItem::Play,
        });
        app.add_systems(Update, handle_main_menu_input);
        app
    }

    #[test]
    fn down_advances_selection() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        app.update();

        let selection = app.world().resource::<MainMenuSelection>();
        assert_eq!(selection.selected, MenuItem::Settings);
    }

    #[test]
    fn up_wraps_selection() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);
        app.update();

        let selection = app.world().resource::<MainMenuSelection>();
        assert_eq!(selection.selected, MenuItem::Quit);
    }

    #[test]
    fn enter_on_play_transitions_to_playing() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("Playing"),
            "expected NextState to contain Playing, got: {next:?}"
        );
    }

    #[test]
    fn enter_on_quit_sends_exit() {
        let mut app = test_app();
        app.world_mut().resource_mut::<MainMenuSelection>().selected = MenuItem::Quit;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        let messages = app.world().resource::<Messages<AppExit>>();
        assert!(
            messages
                .iter_current_update_messages()
                .any(|m| *m == AppExit::Success),
            "expected AppExit::Success message"
        );
    }

    #[test]
    fn enter_on_settings_does_nothing() {
        let mut app = test_app();
        app.world_mut().resource_mut::<MainMenuSelection>().selected = MenuItem::Settings;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        // No state transition
        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            !format!("{next:?}").contains("Playing"),
            "expected no state transition, got: {next:?}"
        );

        // No exit message
        let messages = app.world().resource::<Messages<AppExit>>();
        assert_eq!(
            messages.iter_current_update_messages().count(),
            0,
            "expected no AppExit messages"
        );
    }
}
