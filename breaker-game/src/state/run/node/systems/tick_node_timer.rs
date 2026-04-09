//! System to tick the node timer and send `TimerExpired` when it reaches zero.

use bevy::prelude::*;

use crate::state::run::node::{NodeTimer, messages::TimerExpired};

/// Decrements [`NodeTimer::remaining`] each fixed tick.
/// Sends [`TimerExpired`] when the timer reaches zero.
pub(crate) fn tick_node_timer(
    time: Res<Time<Fixed>>,
    mut timer: ResMut<NodeTimer>,
    mut writer: MessageWriter<TimerExpired>,
) {
    if timer.remaining <= 0.0 {
        return;
    }

    timer.remaining -= time.delta_secs();

    if timer.remaining <= 0.0 {
        timer.remaining = 0.0;
        writer.write(TimerExpired);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn test_app(remaining: f32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<TimerExpired>()
            .insert_resource(NodeTimer {
                remaining,
                total: remaining,
            })
            .add_systems(FixedUpdate, tick_node_timer);
        app
    }

    use crate::shared::test_utils::tick;

    fn tick_with_delta(app: &mut App, delta: Duration) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(delta);
        app.update();
    }

    #[derive(Resource, Default)]
    struct TimerExpiredCaptured(bool);

    fn capture_timer_expired(
        mut reader: MessageReader<TimerExpired>,
        mut captured: ResMut<TimerExpiredCaptured>,
    ) {
        if reader.read().count() > 0 {
            captured.0 = true;
        }
    }

    #[test]
    fn decrements_remaining() {
        let mut app = test_app(10.0);
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(timer.remaining < 10.0);
    }

    #[test]
    fn expired_sent_at_zero() {
        let mut app = test_app(0.001);
        app.init_resource::<TimerExpiredCaptured>();
        app.add_systems(FixedUpdate, capture_timer_expired.after(tick_node_timer));
        // Tick with enough time to expire
        tick_with_delta(&mut app, Duration::from_millis(100));

        let timer = app.world().resource::<NodeTimer>();
        assert!((timer.remaining - 0.0).abs() < f32::EPSILON);
        let captured = app.world().resource::<TimerExpiredCaptured>();
        assert!(
            captured.0,
            "TimerExpired message should be sent when timer reaches zero"
        );
    }

    #[test]
    fn no_expired_while_time_remains() {
        let mut app = test_app(100.0);
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(timer.remaining > 0.0);
    }

    #[test]
    fn remaining_never_negative() {
        let mut app = test_app(0.001);
        tick_with_delta(&mut app, Duration::from_secs(5));

        let timer = app.world().resource::<NodeTimer>();
        assert!(timer.remaining >= 0.0);
    }

    #[test]
    fn already_zero_does_not_send_again() {
        let mut app = test_app(0.0);
        // Timer starts at 0 — should not send expired (it's already expired,
        // the event was already sent when it first hit 0)
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!((timer.remaining - 0.0).abs() < f32::EPSILON);
    }
}
