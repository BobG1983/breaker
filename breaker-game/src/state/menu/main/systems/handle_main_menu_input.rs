//! Main menu keyboard and mouse input handling.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

use crate::{
    input::InputConfig,
    prelude::*,
    state::menu::main::{MENU_ITEMS, MainMenuSelection, MenuItem},
};

/// Handles keyboard and mouse input for the main menu.
///
/// Reads `ButtonInput<KeyCode>` directly instead of `InputActions` because the
/// menu runs in `Update`, while `InputActions` is cleared in `FixedPostUpdate`
/// (between `PreUpdate` and `Update`).
pub(crate) fn handle_main_menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    mut selection: ResMut<MainMenuSelection>,
    mut state_writer: MessageWriter<ChangeState<MenuState>>,
    interaction_query: Query<(&Interaction, &MenuItem), Changed<Interaction>>,
) {
    // Mouse interaction
    for (interaction, item) in &interaction_query {
        match interaction {
            Interaction::Pressed => {
                selection.selected = *item;
                confirm_selection(&selection, &mut state_writer);
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
        confirm_selection(&selection, &mut state_writer);
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
    state_writer: &mut MessageWriter<ChangeState<MenuState>>,
) {
    match selection.selected {
        MenuItem::Play | MenuItem::Quit => {
            state_writer.write(ChangeState::new());
        }
        MenuItem::Settings => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::{app::AppExit, ecs::message::Messages};
    use rantzsoft_stateflow::ChangeState;

    use super::*;

    fn test_app() -> App {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<ButtonInput<KeyCode>>()
            .insert_resource(InputConfig::default())
            .with_message::<AppExit>()
            .with_message::<ChangeState<MenuState>>()
            .insert_resource(MainMenuSelection {
                selected: MenuItem::Play,
            })
            .with_system(Update, handle_main_menu_input)
            .build();
        // Navigate to MenuState
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Menu);
        app.update();
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
    fn enter_on_play_transitions_to_start_game() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let msgs = app.world().resource::<Messages<ChangeState<MenuState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<MenuState> message"
        );
    }

    #[test]
    fn enter_on_quit_sends_change_state() {
        let mut app = test_app();
        app.world_mut().resource_mut::<MainMenuSelection>().selected = MenuItem::Quit;
        press_key(&mut app, KeyCode::Enter);

        let messages = app.world().resource::<Messages<ChangeState<MenuState>>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "expected ChangeState<MenuState> message for quit"
        );
    }

    #[test]
    fn enter_on_settings_does_nothing() {
        let mut app = test_app();
        app.world_mut().resource_mut::<MainMenuSelection>().selected = MenuItem::Settings;
        press_key(&mut app, KeyCode::Enter);

        // No state transition
        let msgs = app.world().resource::<Messages<ChangeState<MenuState>>>();
        assert_eq!(
            msgs.iter_current_update_messages().count(),
            0,
            "expected no ChangeState<MenuState> message"
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
