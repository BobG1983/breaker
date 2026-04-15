//! System to handle timer expiry — lose the run.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

use crate::{
    prelude::*,
    state::run::{
        node::messages::TimerExpired,
        resources::{NodeOutcome, NodeResult},
    },
};

/// When [`TimerExpired`] is received and the run is still in progress, end the run as lost.
///
/// Yields to any transition already queued this frame by `handle_node_cleared`
/// (`run_state.cleared_this_frame`). If the last cell was cleared on the same
/// tick the timer fired, the player wins — clear beats loss.
pub(crate) fn handle_timer_expired(
    mut reader: MessageReader<TimerExpired>,
    mut run_state: ResMut<NodeOutcome>,
    mut writer: MessageWriter<ChangeState<NodeState>>,
) {
    if reader.read().next().is_none() {
        return;
    }

    if run_state.result != NodeResult::InProgress {
        return;
    }

    // Yield to handle_node_cleared if it already queued a transition this frame.
    if run_state.cleared_this_frame {
        return;
    }

    run_state.result = NodeResult::TimerExpired;
    writer.write(ChangeState::new());
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use rantzsoft_stateflow::ChangeState;

    use super::*;

    #[derive(Resource)]
    struct SendTimerExpired(bool);

    fn send_expired(flag: Res<SendTimerExpired>, mut writer: MessageWriter<TimerExpired>) {
        if flag.0 {
            writer.write(TimerExpired);
        }
    }

    fn test_app(result: NodeResult) -> App {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_message::<TimerExpired>()
            .with_message::<ChangeState<NodeState>>()
            .insert_resource(NodeOutcome {
                node_index: 0,
                result,
                ..default()
            })
            .insert_resource(SendTimerExpired(false))
            .with_system(FixedUpdate, (send_expired, handle_timer_expired).chain())
            .build();
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

    #[test]
    fn timer_expired_sets_lost_and_run_end() {
        let mut app = test_app(NodeResult::InProgress);
        app.world_mut().resource_mut::<SendTimerExpired>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::TimerExpired);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<NodeState> message"
        );
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app(NodeResult::InProgress);
        tick(&mut app);

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::InProgress);
    }

    #[test]
    fn timer_expired_yields_to_node_cleared_transition() {
        let mut app = test_app(NodeResult::InProgress);
        // Simulate handle_node_cleared having already queued a transition this frame
        app.world_mut()
            .resource_mut::<NodeOutcome>()
            .cleared_this_frame = true;
        app.world_mut().resource_mut::<SendTimerExpired>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(
            run_state.result,
            NodeResult::InProgress,
            "timer expiry should be silently dropped when a node-cleared transition is already queued"
        );
    }

    #[test]
    fn already_won_ignores_timer_expired() {
        let mut app = test_app(NodeResult::Won);
        app.world_mut().resource_mut::<SendTimerExpired>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::Won);
    }
}
