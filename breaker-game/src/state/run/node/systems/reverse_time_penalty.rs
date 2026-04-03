//! System to reverse time penalties from effect reversal.

use bevy::prelude::*;

use crate::state::run::node::{NodeTimer, messages::ReverseTimePenalty};

/// Reads [`ReverseTimePenalty`] messages and adds seconds back to
/// [`NodeTimer::remaining`], clamping to [`NodeTimer::total`].
///
/// Unlike [`super::apply_time_penalty`], this system does NOT send
/// [`TimerExpired`] — adding time back cannot cause timer expiry.
pub(crate) fn reverse_time_penalty(
    mut reader: MessageReader<ReverseTimePenalty>,
    mut timer: ResMut<NodeTimer>,
) {
    for msg in reader.read() {
        timer.remaining = (timer.remaining + msg.seconds).min(timer.total);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::node::messages::ReverseTimePenalty;

    #[derive(Resource)]
    struct SendReverse(Vec<f32>);

    fn send_reverse(flag: Res<SendReverse>, mut writer: MessageWriter<ReverseTimePenalty>) {
        for &seconds in &flag.0 {
            writer.write(ReverseTimePenalty { seconds });
        }
    }

    fn test_app_with_send(remaining: f32, total: f32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ReverseTimePenalty>()
            .insert_resource(NodeTimer { remaining, total })
            .insert_resource(SendReverse(vec![]))
            .add_systems(FixedUpdate, (send_reverse, reverse_time_penalty).chain());
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 3: adds seconds back to remaining ────────────────

    #[test]
    fn adds_seconds_back_to_remaining() {
        let mut app = test_app_with_send(25.0, 60.0);
        app.world_mut().resource_mut::<SendReverse>().0 = vec![5.0];
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
        app.world_mut().resource_mut::<SendReverse>().0 = vec![0.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 25.0).abs() < f32::EPSILON,
            "remaining should stay at 25.0 with zero-second reverse, got {}",
            timer.remaining
        );
    }

    // ── Behavior 4: clamps remaining to total ─────────────────────

    #[test]
    fn clamps_remaining_to_total() {
        let mut app = test_app_with_send(58.0, 60.0);
        app.world_mut().resource_mut::<SendReverse>().0 = vec![5.0];
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
        app.world_mut().resource_mut::<SendReverse>().0 = vec![5.0];
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
        app.world_mut().resource_mut::<SendReverse>().0 = vec![5.0];
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 5.0).abs() < f32::EPSILON,
            "remaining should be 5.0 after reversing from zero, got {}",
            timer.remaining
        );
    }

    // ── Behavior 6: processes multiple messages in one tick ────────

    #[test]
    fn processes_multiple_messages_in_one_tick() {
        let mut app = test_app_with_send(20.0, 60.0);
        app.world_mut().resource_mut::<SendReverse>().0 = vec![5.0, 3.0];
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
        app.world_mut().resource_mut::<SendReverse>().0 = vec![5.0, 5.0];
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
        // SendReverse default is empty vec — no messages
        tick(&mut app);

        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 25.0).abs() < f32::EPSILON,
            "remaining should stay at 25.0 with no messages, got {}",
            timer.remaining
        );
    }
}
