//! Main menu keyboard and mouse input handling.

use bevy::{app::AppExit, prelude::*};

use crate::{
    input::InputConfig,
    screen::main_menu::{MENU_ITEMS, MainMenuSelection, MenuItem},
    shared::GameState,
};

/// Handles keyboard and mouse input for the main menu.
///
/// Reads `ButtonInput<KeyCode>` directly instead of `InputActions` because the
/// menu runs in `Update`, while `InputActions` is cleared in `FixedPostUpdate`
/// (between `PreUpdate` and `Update`).
pub fn handle_main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
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

    // Keyboard navigation — read raw key state, bypassing InputActions
    if config.menu_down.iter().any(|k| keys.just_pressed(*k)) {
        let current = current_index(&selection);
        let next = (current + 1) % MENU_ITEMS.len();
        selection.selected = MENU_ITEMS[next];
    }

    if config.menu_up.iter().any(|k| keys.just_pressed(*k)) {
        let current = current_index(&selection);
        let next = if current == 0 {
            MENU_ITEMS.len() - 1
        } else {
            current - 1
        };
        selection.selected = MENU_ITEMS[next];
    }

    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
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
    use bevy::{ecs::message::Messages, state::app::StatesPlugin};

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.insert_resource(InputConfig::default());
        app.init_state::<GameState>();
        app.add_message::<AppExit>();
        app.insert_resource(MainMenuSelection {
            selected: MenuItem::Play,
        });
        app.add_systems(Update, handle_main_menu_input);
        app
    }

    /// Simulates a `just_pressed` key event by pressing the key then running
    /// one update. No `InputPlugin` means no `PreUpdate` clear interference.
    fn press_key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
    }

    #[test]
    fn single_down_press_advances_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowDown);

        let selection = app.world().resource::<MainMenuSelection>();
        assert_eq!(selection.selected, MenuItem::Settings);
    }

    #[test]
    fn single_up_press_wraps_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowUp);

        let selection = app.world().resource::<MainMenuSelection>();
        assert_eq!(selection.selected, MenuItem::Quit);
    }

    #[test]
    fn enter_on_play_transitions_to_playing() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

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
        press_key(&mut app, KeyCode::Enter);

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
        press_key(&mut app, KeyCode::Enter);

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
