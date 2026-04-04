//! Handles keyboard input on the pause menu.

use bevy::prelude::*;

use crate::{
    input::InputConfig,
    state::{
        pause::{
            components::{PAUSE_MENU_ITEMS, PauseMenuItem},
            resources::PauseMenuSelection,
        },
        types::RunState,
    },
};

/// Handles keyboard navigation and confirmation on the pause menu.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as main menu).
/// Resume unpauses `Time<Virtual>`. Quit unpauses and sets
/// `RunState::Teardown` to exit the run.
pub(crate) fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    mut selection: ResMut<PauseMenuSelection>,
    mut time: ResMut<Time<Virtual>>,
    mut next_run_phase: ResMut<NextState<RunState>>,
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
                time.unpause();
            }
            PauseMenuItem::Quit => {
                time.unpause();
                next_run_phase.set(RunState::Teardown);
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
    use crate::state::types::{AppState, GameState};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(InputConfig::default())
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .insert_resource(PauseMenuSelection {
                selected: PauseMenuItem::Resume,
            })
            .add_systems(Update, handle_pause_input);
        // Navigate to RunState so NextState<RunState> is available
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        // Pause time to simulate being paused
        app.world_mut().resource_mut::<Time<Virtual>>().pause();
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
    fn confirm_resume_unpauses_time() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let time = app.world().resource::<Time<Virtual>>();
        assert!(
            !time.is_paused(),
            "Time<Virtual> should be unpaused after Resume"
        );
    }

    #[test]
    fn confirm_quit_unpauses_and_sets_teardown() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<PauseMenuSelection>()
            .selected = PauseMenuItem::Quit;
        press_key(&mut app, KeyCode::Enter);

        let time = app.world().resource::<Time<Virtual>>();
        assert!(
            !time.is_paused(),
            "Time<Virtual> should be unpaused after Quit"
        );

        let next = app.world().resource::<NextState<RunState>>();
        assert!(
            format!("{next:?}").contains("Teardown"),
            "expected RunState::Teardown, got: {next:?}"
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
