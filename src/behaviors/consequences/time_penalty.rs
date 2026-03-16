//! Time penalty consequence — observer that translates event into a message.

use bevy::prelude::*;

use crate::run::node::messages::ApplyTimePenalty;

/// Consequence event triggered by bridge systems when time should be subtracted.
#[derive(Event)]
pub struct TimePenaltyRequested {
    /// Seconds to subtract from the node timer.
    pub seconds: f32,
}

/// Observer that handles time penalty — writes [`ApplyTimePenalty`] message.
pub fn handle_time_penalty(
    trigger: On<TimePenaltyRequested>,
    mut writer: MessageWriter<ApplyTimePenalty>,
) {
    writer.write(ApplyTimePenalty {
        seconds: trigger.event().seconds,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct CapturedApplyTimePenalty(Vec<f32>);

    fn capture_apply(
        mut reader: MessageReader<ApplyTimePenalty>,
        mut captured: ResMut<CapturedApplyTimePenalty>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.seconds);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ApplyTimePenalty>();
        app.init_resource::<CapturedApplyTimePenalty>();
        app.add_observer(handle_time_penalty);
        app.add_systems(FixedUpdate, capture_apply);
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
    fn handle_time_penalty_sends_apply_message() {
        let mut app = test_app();

        app.world_mut()
            .commands()
            .trigger(TimePenaltyRequested { seconds: 5.0 });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedApplyTimePenalty>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0] - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn handle_time_penalty_preserves_seconds_value() {
        let mut app = test_app();

        app.world_mut()
            .commands()
            .trigger(TimePenaltyRequested { seconds: 12.5 });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedApplyTimePenalty>();
        assert!((captured.0[0] - 12.5).abs() < f32::EPSILON);
    }
}
