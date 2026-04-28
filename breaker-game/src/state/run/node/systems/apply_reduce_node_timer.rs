//! System to reduce the node timer (consumer of [`ReduceNodeTimer`] messages).

use bevy::prelude::*;

use crate::{
    prelude::*,
    state::run::node::messages::{ReduceNodeTimer, TimerExpired},
};

/// Reads [`ReduceNodeTimer`] messages and subtracts from [`NodeTimer::remaining`].
///
/// Sends [`TimerExpired`] if the timer crosses zero. Skips if the timer is
/// already at zero (idempotent).
pub(crate) fn apply_reduce_node_timer(
    mut reader: MessageReader<ReduceNodeTimer>,
    mut timer: ResMut<NodeTimer>,
    mut writer: MessageWriter<TimerExpired>,
) {
    for msg in reader.read() {
        if timer.remaining <= 0.0 {
            continue;
        }

        timer.remaining -= msg.delta;

        if timer.remaining <= 0.0 {
            timer.remaining = 0.0;
            writer.write(TimerExpired);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct SendReduce(Option<f32>);

    fn send_reduce(flag: Res<SendReduce>, mut writer: MessageWriter<ReduceNodeTimer>) {
        if let Some(delta) = flag.0 {
            writer.write(ReduceNodeTimer { delta });
        }
    }

    #[derive(Resource, Default)]
    struct TimerExpiredCaptured(u32);

    fn capture_timer_expired(
        mut reader: MessageReader<TimerExpired>,
        mut captured: ResMut<TimerExpiredCaptured>,
    ) {
        for _msg in reader.read() {
            captured.0 += 1;
        }
    }

    fn test_app_with_send(remaining: f32) -> App {
        TestAppBuilder::new()
            .with_message::<ReduceNodeTimer>()
            .with_message::<TimerExpired>()
            .insert_resource(NodeTimer {
                remaining,
                total: remaining,
            })
            .insert_resource(SendReduce(None))
            .with_resource::<TimerExpiredCaptured>()
            .with_system(
                FixedUpdate,
                (send_reduce, apply_reduce_node_timer, capture_timer_expired).chain(),
            )
            .build()
    }

    #[test]
    fn subtracts_from_timer() {
        let mut app = test_app_with_send(30.0);
        app.world_mut().resource_mut::<SendReduce>().0 = Some(5.0);
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!((timer.remaining - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clamps_to_zero() {
        let mut app = test_app_with_send(3.0);
        app.world_mut().resource_mut::<SendReduce>().0 = Some(5.0);
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!((timer.remaining - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sends_timer_expired_at_zero() {
        let mut app = test_app_with_send(3.0);
        app.world_mut().resource_mut::<SendReduce>().0 = Some(5.0);
        tick(&mut app);

        let captured = app.world().resource::<TimerExpiredCaptured>();
        assert_eq!(
            captured.0, 1,
            "should send TimerExpired when timer reaches zero"
        );
    }

    #[test]
    fn no_double_expired() {
        let mut app = test_app_with_send(0.0);
        app.world_mut().resource_mut::<SendReduce>().0 = Some(5.0);
        tick(&mut app);

        let captured = app.world().resource::<TimerExpiredCaptured>();
        assert_eq!(
            captured.0, 0,
            "should not send TimerExpired when already at zero"
        );
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app_with_send(30.0);
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!((timer.remaining - 30.0).abs() < f32::EPSILON);
    }
}
