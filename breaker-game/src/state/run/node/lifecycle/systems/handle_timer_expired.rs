//! System to handle timer expiry — lose the run.

use bevy::prelude::*;

use crate::{
    shared::GameState,
    state::run::{
        node::messages::TimerExpired,
        resources::{RunOutcome, RunState},
    },
};

/// When [`TimerExpired`] is received and the run is still in progress, end the run as lost.
///
/// Yields to any transition already queued this frame by `handle_node_cleared`
/// (`run_state.transition_queued`). If the last cell was cleared on the same
/// tick the timer fired, the player wins — clear beats loss.
pub(crate) fn handle_timer_expired(
    mut reader: MessageReader<TimerExpired>,
    mut run_state: ResMut<RunState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if reader.read().next().is_none() {
        return;
    }

    if run_state.outcome != RunOutcome::InProgress {
        return;
    }

    // Yield to handle_node_cleared if it already queued a transition this frame.
    if run_state.transition_queued {
        return;
    }

    run_state.outcome = RunOutcome::TimerExpired;
    next_state.set(GameState::RunEnd);
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    #[derive(Resource)]
    struct SendTimerExpired(bool);

    fn send_expired(flag: Res<SendTimerExpired>, mut writer: MessageWriter<TimerExpired>) {
        if flag.0 {
            writer.write(TimerExpired);
        }
    }

    fn test_app(outcome: RunOutcome) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .add_message::<TimerExpired>()
            .insert_resource(RunState {
                node_index: 0,
                outcome,
                ..default()
            })
            .insert_resource(SendTimerExpired(false))
            .add_systems(FixedUpdate, (send_expired, handle_timer_expired).chain());
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
    fn timer_expired_sets_lost_and_run_end() {
        let mut app = test_app(RunOutcome::InProgress);
        app.world_mut().resource_mut::<SendTimerExpired>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::TimerExpired);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("RunEnd"),
            "expected RunEnd, got: {next:?}"
        );
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app(RunOutcome::InProgress);
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::InProgress);
    }

    #[test]
    fn timer_expired_yields_to_node_cleared_transition() {
        let mut app = test_app(RunOutcome::InProgress);
        // Simulate handle_node_cleared having already queued a transition this frame
        app.world_mut().resource_mut::<RunState>().transition_queued = true;
        app.world_mut().resource_mut::<SendTimerExpired>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(
            run_state.outcome,
            RunOutcome::InProgress,
            "timer expiry should be silently dropped when a node-cleared transition is already queued"
        );
    }

    #[test]
    fn already_won_ignores_timer_expired() {
        let mut app = test_app(RunOutcome::Won);
        app.world_mut().resource_mut::<SendTimerExpired>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::Won);
    }
}
