//! Handles keyboard input on the pause menu.

use bevy::prelude::*;
use rantzsoft_lifecycle::ChangeState;

use crate::{
    input::InputConfig,
    state::{
        pause::{
            components::{PAUSE_MENU_ITEMS, PauseMenuItem},
            resources::PauseMenuSelection,
        },
        run::resources::{NodeOutcome, NodeResult},
        types::NodeState,
    },
};

/// Handles keyboard navigation and confirmation on the pause menu.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as main menu).
/// Resume unpauses `Time<Virtual>`. Quit unpauses, sets
/// `NodeOutcome.result` to `NodeResult::Quit`, and writes a
/// `ChangeState<NodeState>` message to exit the node.
pub(crate) fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    mut selection: ResMut<PauseMenuSelection>,
    mut time: ResMut<Time<Virtual>>,
    mut writer: MessageWriter<ChangeState<NodeState>>,
    mut node_outcome: ResMut<NodeOutcome>,
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
                node_outcome.result = NodeResult::Quit;
                writer.write(ChangeState::new());
                time.unpause();
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
    use bevy::{ecs::message::Messages, state::app::StatesPlugin};
    use rantzsoft_lifecycle::ChangeState;

    use super::*;
    use crate::state::{
        run::resources::{NodeOutcome, NodeResult},
        types::{AppState, GameState, NodeState, RunState},
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(InputConfig::default())
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_message::<ChangeState<NodeState>>()
            .insert_resource(NodeOutcome::default())
            .insert_resource(PauseMenuSelection {
                selected: PauseMenuItem::Resume,
            })
            .add_systems(Update, handle_pause_input);
        // Navigate to NodeState so sub-states are active
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
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

    // ── Navigation tests (existing, preserved) ──────────────────────────

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

    // ── Behavior 1: Quit writes ChangeState<NodeState> message ──────────

    #[test]
    fn confirm_quit_writes_change_state_message() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<PauseMenuSelection>()
            .selected = PauseMenuItem::Quit;
        press_key(&mut app, KeyCode::Enter);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<NodeState> message after quit"
        );
    }

    #[test]
    fn confirm_resume_does_not_write_change_state_message() {
        // Use a fresh app — verify Resume path writes zero messages.
        // confirm_quit_writes_change_state_message proves the system CAN write;
        // this test verifies Resume does not.
        let mut app = test_app();
        // Selection defaults to Resume
        press_key(&mut app, KeyCode::Enter);

        let resume_count = app
            .world()
            .resource::<Messages<ChangeState<NodeState>>>()
            .iter_current_update_messages()
            .count();
        assert_eq!(
            resume_count, 0,
            "Resume should not write ChangeState<NodeState>"
        );
    }

    // ── Behavior 2: Quit sets NodeOutcome.result to NodeResult::Quit ────

    #[test]
    fn confirm_quit_sets_node_outcome_to_quit() {
        let mut app = test_app();
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
        app.world_mut()
            .resource_mut::<PauseMenuSelection>()
            .selected = PauseMenuItem::Quit;
        press_key(&mut app, KeyCode::Enter);

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(
            outcome.result,
            NodeResult::Quit,
            "Quit action should set NodeResult::Quit"
        );
        assert_eq!(
            outcome.node_index, 2,
            "node_index should be unchanged after quit"
        );
        assert!(
            !outcome.transition_queued,
            "transition_queued should remain false"
        );
    }

    #[test]
    fn confirm_quit_overrides_prior_timer_expired_result() {
        let mut app = test_app();
        app.world_mut().resource_mut::<NodeOutcome>().result = NodeResult::TimerExpired;
        app.world_mut()
            .resource_mut::<PauseMenuSelection>()
            .selected = PauseMenuItem::Quit;
        press_key(&mut app, KeyCode::Enter);

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(
            outcome.result,
            NodeResult::Quit,
            "Quit should override any prior NodeResult (including TimerExpired)"
        );
    }

    // ── Behavior 3: Quit unpauses Time<Virtual> ─────────────────────────

    #[test]
    fn confirm_quit_unpauses_time() {
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
    }

    #[test]
    fn confirm_quit_unpauses_already_unpaused_time() {
        let mut app = test_app();
        // Unpause before quit — should not panic
        app.world_mut().resource_mut::<Time<Virtual>>().unpause();
        app.world_mut()
            .resource_mut::<PauseMenuSelection>()
            .selected = PauseMenuItem::Quit;
        press_key(&mut app, KeyCode::Enter);

        let time = app.world().resource::<Time<Virtual>>();
        assert!(!time.is_paused(), "Time<Virtual> should remain unpaused");
    }

    // ── Behavior 4: Resume does not touch NodeOutcome ───────────────────

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
    fn confirm_resume_does_not_change_node_outcome() {
        let mut app = test_app();
        // Pre-set NodeOutcome to a non-default value so we can detect if
        // the system overwrites it.
        app.world_mut().resource_mut::<NodeOutcome>().result = NodeResult::TimerExpired;

        // Selection defaults to Resume
        press_key(&mut app, KeyCode::Enter);

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(
            outcome.result,
            NodeResult::TimerExpired,
            "Resume should not modify NodeOutcome.result — it must remain TimerExpired"
        );
    }
}
