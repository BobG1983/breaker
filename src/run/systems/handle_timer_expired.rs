//! System to handle timer expiry — lose the run.

use bevy::prelude::*;

use crate::{
    run::{
        messages::TimerExpired,
        resources::{RunOutcome, RunState},
    },
    shared::GameState,
};

/// When [`TimerExpired`] is received and the run is still in progress, end the run as lost.
pub fn handle_timer_expired(
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

    run_state.outcome = RunOutcome::Lost;
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
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<GameState>();
        app.add_message::<TimerExpired>();
        app.insert_resource(RunState {
            node_index: 0,
            outcome,
        });
        app.insert_resource(SendTimerExpired(false));
        app.add_systems(FixedUpdate, (send_expired, handle_timer_expired).chain());
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
        assert_eq!(run_state.outcome, RunOutcome::Lost);

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
    fn already_won_ignores_timer_expired() {
        let mut app = test_app(RunOutcome::Won);
        app.world_mut().resource_mut::<SendTimerExpired>().0 = true;
        tick(&mut app);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::Won);
    }
}
