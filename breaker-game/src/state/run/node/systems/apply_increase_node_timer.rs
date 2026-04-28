//! System to increase the node timer (consumer of [`IncreaseNodeTimer`] messages).

use bevy::prelude::*;

use crate::{prelude::*, state::run::node::messages::IncreaseNodeTimer};

/// Reads [`IncreaseNodeTimer`] messages and adds delta back to
/// [`NodeTimer::remaining`], clamping to [`NodeTimer::total`].
///
/// Unlike [`super::apply_reduce_node_timer`], this system does NOT send
/// [`TimerExpired`] — adding time back cannot cause timer expiry.
pub(crate) fn apply_increase_node_timer(
    mut reader: MessageReader<IncreaseNodeTimer>,
    mut timer: ResMut<NodeTimer>,
) {
    for msg in reader.read() {
        timer.remaining = (timer.remaining + msg.delta).min(timer.total);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::node::messages::IncreaseNodeTimer;

    #[derive(Resource)]
    struct SendIncrease(Vec<f32>);

    fn send_increase(flag: Res<SendIncrease>, mut writer: MessageWriter<IncreaseNodeTimer>) {
        for &delta in &flag.0 {
            writer.write(IncreaseNodeTimer { delta });
        }
    }

    fn test_app_with_send(remaining: f32, total: f32) -> App {
        TestAppBuilder::new()
            .with_message::<IncreaseNodeTimer>()
            .insert_resource(NodeTimer { remaining, total })
            .insert_resource(SendIncrease(vec![]))
            .with_system(
                FixedUpdate,
                (send_increase, apply_increase_node_timer).chain(),
            )
            .build()
    }

    // ── Behavior 3: adds seconds back to remaining ────────────────

    #[test]
    fn adds_seconds_back_to_remaining() {
        let mut app = test_app_with_send(25.0, 60.0);
        app.world_mut().resource_mut::<SendIncrease>().0 = vec![5.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 30.0).abs() < f32::EPSILON,
            "remaining should be 30.0, got {}",
            timer.remaining
        );
    }

    #[test]
    fn zero_seconds_does_not_change_remaining() {
        let mut app = test_app_with_send(25.0, 60.0);
        app.world_mut().resource_mut::<SendIncrease>().0 = vec![0.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 25.0).abs() < f32::EPSILON,
            "remaining should stay at 25.0 with zero-delta increase, got {}",
            timer.remaining
        );
    }

    // ── Behavior 4: clamps remaining to total ─────────────────────

    #[test]
    fn clamps_remaining_to_total() {
        let mut app = test_app_with_send(58.0, 60.0);
        app.world_mut().resource_mut::<SendIncrease>().0 = vec![5.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 60.0).abs() < f32::EPSILON,
            "remaining should clamp to total (60.0), got {}",
            timer.remaining
        );
    }

    #[test]
    fn at_total_remains_at_total() {
        let mut app = test_app_with_send(60.0, 60.0);
        app.world_mut().resource_mut::<SendIncrease>().0 = vec![5.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 60.0).abs() < f32::EPSILON,
            "remaining should stay at total (60.0), got {}",
            timer.remaining
        );
    }

    // ── Behavior 5: restores time from zero ───────────────────────

    #[test]
    fn restores_time_from_zero() {
        let mut app = test_app_with_send(0.0, 60.0);
        app.world_mut().resource_mut::<SendIncrease>().0 = vec![5.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 5.0).abs() < f32::EPSILON,
            "remaining should be 5.0 after increasing from zero, got {}",
            timer.remaining
        );
    }

    // ── Behavior 6: processes multiple messages in one tick ────────

    #[test]
    fn processes_multiple_messages_in_one_tick() {
        let mut app = test_app_with_send(20.0, 60.0);
        app.world_mut().resource_mut::<SendIncrease>().0 = vec![5.0, 3.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 28.0).abs() < f32::EPSILON,
            "remaining should be 28.0 (20.0 + 5.0 + 3.0), got {}",
            timer.remaining
        );
    }

    #[test]
    fn multiple_messages_clamp_to_total() {
        let mut app = test_app_with_send(55.0, 60.0);
        app.world_mut().resource_mut::<SendIncrease>().0 = vec![5.0, 5.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 60.0).abs() < f32::EPSILON,
            "remaining should clamp to total (60.0), got {}",
            timer.remaining
        );
    }

    // ── Behavior 7: no messages does nothing ──────────────────────

    #[test]
    fn no_message_no_change() {
        let mut app = test_app_with_send(25.0, 60.0);
        // SendIncrease starts as an empty vec — no messages
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 25.0).abs() < f32::EPSILON,
            "remaining should stay at 25.0 with no messages, got {}",
            timer.remaining
        );
    }
}
