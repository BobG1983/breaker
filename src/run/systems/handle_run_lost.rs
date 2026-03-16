//! System to handle run lost — set outcome and transition to `RunEnd`.

use bevy::prelude::*;

use crate::{
    run::{
        messages::RunLost,
        resources::{RunOutcome, RunState},
    },
    shared::GameState,
};

/// When [`RunLost`] is received, sets the run outcome to lost and transitions
/// to [`GameState::RunEnd`].
pub fn handle_run_lost(
    mut reader: MessageReader<RunLost>,
    mut run_state: ResMut<RunState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for _msg in reader.read() {
        if run_state.outcome == RunOutcome::InProgress {
            run_state.outcome = RunOutcome::Lost;
            next_state.set(GameState::RunEnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    #[derive(Resource)]
    struct SendRunLost(bool);

    fn send_run_lost(flag: Res<SendRunLost>, mut writer: MessageWriter<RunLost>) {
        if flag.0 {
            writer.write(RunLost);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<GameState>();
        app.add_message::<RunLost>();
        app.insert_resource(RunState {
            node_index: 0,
            outcome: RunOutcome::InProgress,
            ..default()
        });
        app.insert_resource(SendRunLost(false));
        app.add_systems(
            FixedUpdate,
            (send_run_lost.before(handle_run_lost), handle_run_lost),
        );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn run_lost_sets_outcome_and_transitions() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SendRunLost>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::Lost);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("RunEnd"),
            "expected RunEnd, got: {next:?}"
        );
    }

    #[test]
    fn run_lost_ignored_when_already_won() {
        let mut app = test_app();
        app.world_mut().resource_mut::<RunState>().outcome = RunOutcome::Won;
        app.world_mut().resource_mut::<SendRunLost>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::Won);
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app();
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::InProgress);
    }
}
