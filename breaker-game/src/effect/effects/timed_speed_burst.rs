//! Timed speed burst chip effect — temporary velocity boost on bolt.
//!
//! Observes [`TimedSpeedBurstFired`] and inserts or refreshes [`TimedSpeedBurstActive`]
//! on the bolt entity. [`tick_timed_speed_burst`] decrements the remaining time each
//! fixed tick and restores the bolt's speed when the burst expires.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::{Bolt, BoltMaxSpeed},
    effect::typed_events::TimedSpeedBurstFired,
};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks a bolt as having an active speed burst. The `remaining` field counts down
/// each tick; `speed_mult` records the multiplier for restoration on expiry.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct TimedSpeedBurstActive {
    /// Seconds of burst time remaining.
    pub remaining: f32,
    /// The multiplier currently applied to the bolt's velocity.
    pub speed_mult: f32,
}

// ---------------------------------------------------------------------------
// Observer — inserts / refreshes TimedSpeedBurstActive
// ---------------------------------------------------------------------------

/// Observer: handles speed burst activation — inserts or refreshes
/// [`TimedSpeedBurstActive`] on the bolt entity, scaling velocity and
/// clamping to [`BoltMaxSpeed`].
pub(crate) fn handle_timed_speed_burst(
    trigger: On<TimedSpeedBurstFired>,
    mut query: Query<
        (
            &mut Velocity2D,
            &BoltMaxSpeed,
            Option<&mut TimedSpeedBurstActive>,
        ),
        With<Bolt>,
    >,
    mut commands: Commands,
) {
    let event = trigger.event();
    let Some(bolt_entity) = event.bolt else {
        return;
    };
    let Ok((mut vel, max_speed, existing)) = query.get_mut(bolt_entity) else {
        return;
    };

    if let Some(mut active) = existing {
        if event.speed_mult > active.speed_mult {
            // Stronger multiplier: undo old, apply new
            let speed = vel.0.length();
            let base = speed / active.speed_mult;
            let new_speed = (base * event.speed_mult).min(max_speed.0);
            vel.0 = vel.0.normalize_or_zero() * new_speed;
            active.speed_mult = event.speed_mult;
        }
        // Always refresh duration
        active.remaining = event.duration_secs;
    } else {
        // New burst
        let speed = vel.0.length();
        let new_speed = (speed * event.speed_mult).min(max_speed.0);
        vel.0 = vel.0.normalize_or_zero() * new_speed;
        commands.entity(bolt_entity).insert(TimedSpeedBurstActive {
            remaining: event.duration_secs,
            speed_mult: event.speed_mult,
        });
    }
}

// ---------------------------------------------------------------------------
// Tick system — decrements remaining and restores speed on expiry
// ---------------------------------------------------------------------------

/// Decrements `TimedSpeedBurstActive::remaining` each fixed tick.
/// On expiry (<= 0.0), restores the bolt's velocity by dividing by `speed_mult`
/// and removes the component.
pub(crate) fn tick_timed_speed_burst(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TimedSpeedBurstActive, &mut Velocity2D)>,
) {
    for (entity, mut active, mut vel) in &mut query {
        active.remaining -= time.delta_secs();
        if active.remaining <= 0.0 {
            let speed = vel.0.length();
            let restored = speed / active.speed_mult;
            vel.0 = vel.0.normalize_or_zero() * restored;
            commands.entity(entity).remove::<TimedSpeedBurstActive>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltMaxSpeed};

    // --- Test infrastructure ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_timed_speed_burst)
            .add_systems(FixedUpdate, tick_timed_speed_burst);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_bolt(app: &mut App, velocity: Vec2, max_speed: f32) -> Entity {
        app.world_mut()
            .spawn((Bolt, Velocity2D(velocity), BoltMaxSpeed(max_speed)))
            .id()
    }

    fn trigger_burst(app: &mut App, speed_mult: f32, duration_secs: f32, bolt: Option<Entity>) {
        app.world_mut().commands().trigger(TimedSpeedBurstFired {
            speed_mult,
            duration_secs,
            bolt,
            source_chip: Some("Test Overclock".to_owned()),
        });
        app.world_mut().flush();
    }

    // =========================================================================
    // Behavior 17: handle_timed_speed_burst inserts TimedSpeedBurstActive
    // =========================================================================

    #[test]
    fn handle_timed_speed_burst_inserts_active_and_scales_velocity() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, Vec2::new(0.0, 400.0), 800.0);

        trigger_burst(&mut app, 1.5, 3.0, Some(bolt));

        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        let expected_speed = 400.0 * 1.5;
        assert!(
            (vel.0.length() - expected_speed).abs() < 1.0,
            "velocity magnitude should be ~{expected_speed}, got {}",
            vel.0.length()
        );

        let burst = app
            .world()
            .get::<TimedSpeedBurstActive>(bolt)
            .expect("bolt should have TimedSpeedBurstActive");
        assert!(
            (burst.remaining - 3.0).abs() < f32::EPSILON,
            "remaining should be 3.0, got {}",
            burst.remaining
        );
        assert!(
            (burst.speed_mult - 1.5).abs() < f32::EPSILON,
            "speed_mult should be 1.5, got {}",
            burst.speed_mult
        );
    }

    // =========================================================================
    // Behavior 18: handle_timed_speed_burst clamps to BoltMaxSpeed
    // =========================================================================

    #[test]
    fn handle_timed_speed_burst_clamps_to_max_speed() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, Vec2::new(0.0, 600.0), 800.0);

        trigger_burst(&mut app, 1.5, 3.0, Some(bolt));

        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        // 600 * 1.5 = 900 > 800, should clamp to 800
        assert!(
            (vel.0.length() - 800.0).abs() < 1.0,
            "velocity should be clamped to BoltMaxSpeed 800.0, got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 19: refresh takes stronger multiplier
    // =========================================================================

    #[test]
    fn handle_timed_speed_burst_refresh_takes_stronger_multiplier() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 520.0)),
                BoltMaxSpeed(800.0),
                TimedSpeedBurstActive {
                    remaining: 1.0,
                    speed_mult: 1.3,
                },
            ))
            .id();

        trigger_burst(&mut app, 1.5, 3.0, Some(bolt));

        let burst = app
            .world()
            .get::<TimedSpeedBurstActive>(bolt)
            .expect("bolt should have TimedSpeedBurstActive");
        assert!(
            (burst.remaining - 3.0).abs() < f32::EPSILON,
            "remaining should be refreshed to 3.0, got {}",
            burst.remaining
        );
        assert!(
            (burst.speed_mult - 1.5).abs() < f32::EPSILON,
            "speed_mult should be upgraded to 1.5, got {}",
            burst.speed_mult
        );

        // Velocity should be rescaled: (520.0 / 1.3) * 1.5 = 600.0
        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 600.0).abs() < 1.0,
            "velocity should be rescaled from old mult to new: (520/1.3)*1.5 = 600, got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 20: weaker refresh keeps higher multiplier
    // =========================================================================

    #[test]
    fn handle_timed_speed_burst_weaker_refresh_keeps_higher_multiplier() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 600.0)),
                BoltMaxSpeed(800.0),
                TimedSpeedBurstActive {
                    remaining: 1.0,
                    speed_mult: 1.5,
                },
            ))
            .id();

        // Fire a weaker burst
        trigger_burst(&mut app, 1.3, 3.0, Some(bolt));

        let burst = app
            .world()
            .get::<TimedSpeedBurstActive>(bolt)
            .expect("bolt should have TimedSpeedBurstActive");
        assert!(
            (burst.remaining - 3.0).abs() < f32::EPSILON,
            "remaining should be refreshed to 3.0, got {}",
            burst.remaining
        );
        assert!(
            (burst.speed_mult - 1.5).abs() < f32::EPSILON,
            "speed_mult should stay at 1.5 (stronger), got {}",
            burst.speed_mult
        );

        // Velocity should be unchanged
        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 600.0).abs() < 1.0,
            "velocity should be unchanged at 600.0, got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 21: tick_timed_speed_burst decrements remaining
    // =========================================================================

    #[test]
    fn tick_timed_speed_burst_decrements_remaining() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, tick_timed_speed_burst);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 520.0)),
                BoltMaxSpeed(800.0),
                TimedSpeedBurstActive {
                    remaining: 2.0,
                    speed_mult: 1.3,
                },
            ))
            .id();

        tick(&mut app);

        let burst = app
            .world()
            .get::<TimedSpeedBurstActive>(bolt)
            .expect("TimedSpeedBurstActive should still exist after one tick");
        // dt = 1/64 = 0.015625
        let expected = 2.0 - (1.0 / 64.0);
        assert!(
            (burst.remaining - expected).abs() < 0.01,
            "remaining should decrease by dt (~1/64), expected ~{expected:.4}, got {:.4}",
            burst.remaining
        );
    }

    // =========================================================================
    // Behavior 22: tick_timed_speed_burst removes on expiry and restores speed
    // =========================================================================

    #[test]
    fn tick_timed_speed_burst_removes_on_expiry_and_restores_speed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, tick_timed_speed_burst);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 600.0)),
                BoltMaxSpeed(800.0),
                TimedSpeedBurstActive {
                    remaining: 0.01,
                    speed_mult: 1.5,
                },
            ))
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<TimedSpeedBurstActive>(bolt).is_none(),
            "TimedSpeedBurstActive should be removed when remaining drops to <= 0.0"
        );

        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should still have Velocity2D");
        // Restored speed: 600.0 / 1.5 = 400.0
        assert!(
            (vel.0.length() - 400.0).abs() < 1.0,
            "velocity should be restored to ~400.0 (600/1.5), got {}",
            vel.0.length()
        );
    }
}
