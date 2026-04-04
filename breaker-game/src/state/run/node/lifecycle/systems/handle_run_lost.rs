//! System to handle run lost — set outcome and transition to `RunEnd`.

use bevy::prelude::*;

use crate::state::{
    run::{
        messages::RunLost,
        resources::{NodeOutcome, NodeResult},
    },
    types::NodeState,
};

/// When [`RunLost`] is received, sets the run outcome to lost and transitions
/// to [`NodeState::AnimateOut`]. The teardown router reads `NodeOutcome` to
/// route to `RunEnd`.
pub(crate) fn handle_run_lost(
    mut reader: MessageReader<RunLost>,
    mut run_state: ResMut<NodeOutcome>,
    mut next_state: ResMut<NextState<NodeState>>,
) {
    for _msg in reader.read() {
        if run_state.result == NodeResult::InProgress {
            run_state.result = NodeResult::LivesDepleted;
            next_state.set(NodeState::AnimateOut);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::state::types::{AppState, GameState, RunState};

    #[derive(Resource)]
    struct SendRunLost(bool);

    fn send_run_lost(flag: Res<SendRunLost>, mut writer: MessageWriter<RunLost>) {
        if flag.0 {
            writer.write(RunLost);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_message::<RunLost>()
            .insert_resource(NodeOutcome {
                node_index: 0,
                result: NodeResult::InProgress,
                ..default()
            })
            .insert_resource(SendRunLost(false))
            .add_systems(
                FixedUpdate,
                (send_run_lost.before(handle_run_lost), handle_run_lost),
            );
        // Navigate to NodeState
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

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::LivesDepleted);

        let next = app.world().resource::<NextState<NodeState>>();
        assert!(
            format!("{next:?}").contains("AnimateOut"),
            "expected AnimateOut, got: {next:?}"
        );
    }

    #[test]
    fn run_lost_ignored_when_already_won() {
        let mut app = test_app();
        app.world_mut().resource_mut::<NodeOutcome>().result = NodeResult::Won;
        app.world_mut().resource_mut::<SendRunLost>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::Won);
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app();
        tick(&mut app);

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::InProgress);
    }
}
