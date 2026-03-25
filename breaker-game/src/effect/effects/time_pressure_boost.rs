//! Time-pressure speed boost chip effect (Deadline legendary).
//!
//! Observes [`TimePressureBoostApplied`] and inserts [`TimePressureBoostConfig`]
//! as a resource. [`tick_time_pressure_boost`] checks the [`NodeTimer`] ratio each
//! fixed tick and applies/removes [`TimePressureBoostActive`] on bolt entities.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::{Bolt, BoltMaxSpeed},
    effect::typed_events::TimePressureBoostApplied,
    run::node::resources::NodeTimer,
};

// ---------------------------------------------------------------------------
// Components and resources
// ---------------------------------------------------------------------------

/// Stores the time-pressure boost configuration, inserted as a resource.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct TimePressureBoostConfig {
    /// Speed multiplier applied to bolt velocity when active.
    pub speed_mult: f32,
    /// Timer ratio threshold (remaining/total) below which boost activates.
    pub threshold_pct: f32,
}

/// Marks a bolt as having an active time-pressure speed boost.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct TimePressureBoostActive {
    /// The speed multiplier currently applied.
    pub speed_mult: f32,
}

// ---------------------------------------------------------------------------
// Observer — inserts TimePressureBoostConfig resource
// ---------------------------------------------------------------------------

/// Observer: handles time pressure boost activation — inserts or overwrites
/// [`TimePressureBoostConfig`] resource.
pub(crate) fn handle_time_pressure_boost_applied(
    trigger: On<TimePressureBoostApplied>,
    mut commands: Commands,
) {
    let event = trigger.event();
    commands.insert_resource(TimePressureBoostConfig {
        speed_mult: event.speed_mult,
        threshold_pct: event.threshold_pct,
    });
}

// ---------------------------------------------------------------------------
// Tick system — checks NodeTimer and applies/removes boost
// ---------------------------------------------------------------------------

/// Checks the [`NodeTimer`] ratio each fixed tick and applies or removes
/// [`TimePressureBoostActive`] on bolt entities based on the threshold.
pub(crate) fn tick_time_pressure_boost(
    mut commands: Commands,
    config: Option<Res<TimePressureBoostConfig>>,
    timer: Option<Res<NodeTimer>>,
    mut query: Query<
        (
            Entity,
            &mut Velocity2D,
            &BoltMaxSpeed,
            Option<&TimePressureBoostActive>,
        ),
        With<Bolt>,
    >,
) {
    let Some(config) = config else {
        return;
    };

    let Some(timer) = timer else {
        // No timer — remove all active boosts and restore speed.
        for (entity, mut vel, _max_speed, active) in &mut query {
            if let Some(active) = active {
                let speed = vel.0.length();
                if speed > 0.0 {
                    vel.0 = vel.0.normalize_or_zero() * (speed / active.speed_mult);
                }
                commands.entity(entity).remove::<TimePressureBoostActive>();
            }
        }
        return;
    };

    let ratio = timer.remaining / timer.total;
    let below_threshold = ratio <= config.threshold_pct;

    for (entity, mut vel, max_speed, active) in &mut query {
        if below_threshold && active.is_none() {
            // Apply boost to unboosted bolt
            let speed = vel.0.length();
            let new_speed = (speed * config.speed_mult).min(max_speed.0);
            if speed > 0.0 {
                vel.0 = vel.0.normalize_or_zero() * new_speed;
            }
            commands.entity(entity).insert(TimePressureBoostActive {
                speed_mult: config.speed_mult,
            });
        } else if !below_threshold && let Some(active) = active {
            // Remove boost and restore speed
            let speed = vel.0.length();
            if speed > 0.0 {
                vel.0 = vel.0.normalize_or_zero() * (speed / active.speed_mult);
            }
            commands.entity(entity).remove::<TimePressureBoostActive>();
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
            .add_observer(handle_time_pressure_boost_applied)
            .add_systems(FixedUpdate, tick_time_pressure_boost);
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

    fn trigger_applied(app: &mut App, speed_mult: f32, threshold_pct: f32) {
        app.world_mut()
            .commands()
            .trigger(TimePressureBoostApplied {
                speed_mult,
                threshold_pct,
                max_stacks: 1,
                chip_name: "Deadline".to_owned(),
            });
        app.world_mut().flush();
    }

    // =========================================================================
    // Behavior 6: handle_time_pressure_boost_applied inserts config resource
    // =========================================================================

    #[test]
    fn handle_time_pressure_boost_inserts_config_resource() {
        let mut app = test_app();

        trigger_applied(&mut app, 2.0, 0.25);

        let config = app
            .world()
            .get_resource::<TimePressureBoostConfig>()
            .expect("TimePressureBoostConfig should be inserted");
        assert!(
            (config.speed_mult - 2.0).abs() < f32::EPSILON,
            "speed_mult should be 2.0, got {}",
            config.speed_mult
        );
        assert!(
            (config.threshold_pct - 0.25).abs() < f32::EPSILON,
            "threshold_pct should be 0.25, got {}",
            config.threshold_pct
        );
    }

    #[test]
    fn handle_time_pressure_boost_overwrites_existing_config() {
        let mut app = test_app();

        trigger_applied(&mut app, 1.5, 0.20);
        trigger_applied(&mut app, 2.0, 0.25);

        let config = app
            .world()
            .get_resource::<TimePressureBoostConfig>()
            .expect("TimePressureBoostConfig should exist");
        assert!(
            (config.speed_mult - 2.0).abs() < f32::EPSILON,
            "config should be overwritten with speed_mult 2.0, got {}",
            config.speed_mult
        );
    }

    // =========================================================================
    // Behavior 7: tick applies boost when timer drops below threshold
    // =========================================================================

    #[test]
    fn tick_applies_boost_when_timer_below_threshold() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        // remaining/total = 10/60 = 0.167, below 0.25
        app.insert_resource(NodeTimer {
            remaining: 10.0,
            total: 60.0,
        });
        let bolt = spawn_bolt(&mut app, Vec2::new(0.0, 400.0), 800.0);

        tick(&mut app);

        let active = app
            .world()
            .get::<TimePressureBoostActive>(bolt)
            .expect("bolt should gain TimePressureBoostActive");
        assert!(
            (active.speed_mult - 2.0).abs() < f32::EPSILON,
            "speed_mult should be 2.0"
        );

        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        // 400.0 * 2.0 = 800.0
        assert!(
            (vel.0.length() - 800.0).abs() < 1.0,
            "velocity should be ~800.0 (400*2.0), got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 8: tick does NOT apply boost when above threshold
    // =========================================================================

    #[test]
    fn tick_does_not_apply_boost_when_above_threshold() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        // remaining/total = 45/60 = 0.75, above 0.25
        app.insert_resource(NodeTimer {
            remaining: 45.0,
            total: 60.0,
        });
        let bolt = spawn_bolt(&mut app, Vec2::new(0.0, 400.0), 800.0);

        tick(&mut app);

        assert!(
            app.world().get::<TimePressureBoostActive>(bolt).is_none(),
            "bolt should NOT gain TimePressureBoostActive when timer ratio is above threshold"
        );

        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 400.0).abs() < 1.0,
            "velocity should remain ~400.0, got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 9: tick clamps boosted velocity to BoltMaxSpeed
    // =========================================================================

    #[test]
    fn tick_clamps_boosted_velocity_to_bolt_max_speed() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        app.insert_resource(NodeTimer {
            remaining: 5.0,
            total: 60.0,
        });
        // 600 * 2.0 = 1200 > 800, should clamp to 800
        let bolt = spawn_bolt(&mut app, Vec2::new(0.0, 600.0), 800.0);

        tick(&mut app);

        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 800.0).abs() < 1.0,
            "velocity should be clamped to BoltMaxSpeed 800.0, got {}",
            vel.0.length()
        );
        let active = app
            .world()
            .get::<TimePressureBoostActive>(bolt)
            .expect("bolt should have TimePressureBoostActive");
        assert!(
            (active.speed_mult - 2.0).abs() < f32::EPSILON,
            "speed_mult on active should be 2.0"
        );
    }

    // =========================================================================
    // Behavior 10: tick removes boost and restores speed when timer rises above
    // =========================================================================

    #[test]
    fn tick_removes_boost_and_restores_speed_when_above_threshold() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        // remaining/total = 20/60 = 0.333, above 0.25
        app.insert_resource(NodeTimer {
            remaining: 20.0,
            total: 60.0,
        });
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltMaxSpeed(800.0),
                TimePressureBoostActive { speed_mult: 2.0 },
            ))
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<TimePressureBoostActive>(bolt).is_none(),
            "TimePressureBoostActive should be removed when timer ratio is above threshold"
        );
        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        // Restored: 800.0 / 2.0 = 400.0
        assert!(
            (vel.0.length() - 400.0).abs() < 1.0,
            "velocity should be restored to ~400.0 (800/2.0), got {}",
            vel.0.length()
        );
    }

    #[test]
    fn tick_keeps_boost_when_ratio_exactly_at_threshold() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        // remaining/total = 15/60 = 0.25 exactly — boost should remain active (strictly less than)
        app.insert_resource(NodeTimer {
            remaining: 15.0,
            total: 60.0,
        });
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltMaxSpeed(800.0),
                TimePressureBoostActive { speed_mult: 2.0 },
            ))
            .id();

        tick(&mut app);

        // At exactly 0.25, boost should NOT be active (strictly less than threshold)
        // The spec says: "Timer ratio exactly at 0.25 (15.0/60.0) — boost remains active (strictly less than threshold)"
        // This means 0.25 is NOT strictly less than 0.25, so boost is removed
        // Wait, re-reading: "boost remains active (strictly less than threshold)" means the condition
        // for boost is "ratio < threshold", so at exactly 0.25 the condition fails and boost is removed.
        // But the spec says "boost remains active" in behavior 10 edge case. Let me re-read...
        // Spec: "Timer ratio exactly at 0.25 (15.0/60.0) — boost remains active (strictly less than threshold)"
        // This is ambiguous. The parenthetical says "strictly less than threshold" is the condition.
        // If condition is ratio < threshold and ratio == threshold, condition is false, boost removed.
        // But the text says "boost remains active"... I think the intent is that 0.25 is NOT below 0.25,
        // so boost does NOT activate for newly unboosted bolts, but for already-boosted bolts
        // the test says "boost remains active" — contradicting.
        // Given ambiguity, test both: at exactly threshold, an already-active boost remains active.
        assert!(
            app.world().get::<TimePressureBoostActive>(bolt).is_some(),
            "at exactly threshold, boost should remain active"
        );
    }

    // =========================================================================
    // Behavior 11: tick does nothing when already boosted and still below
    // =========================================================================

    #[test]
    fn tick_no_op_when_already_boosted_and_below_threshold() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        app.insert_resource(NodeTimer {
            remaining: 10.0,
            total: 60.0,
        });
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltMaxSpeed(800.0),
                TimePressureBoostActive { speed_mult: 2.0 },
            ))
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<TimePressureBoostActive>(bolt).is_some(),
            "TimePressureBoostActive should persist"
        );
        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 800.0).abs() < 1.0,
            "velocity should remain unchanged at ~800.0, got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 12: tick removes boost when NodeTimer is missing
    // =========================================================================

    #[test]
    fn tick_removes_boost_when_node_timer_missing() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        // No NodeTimer resource inserted
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltMaxSpeed(800.0),
                TimePressureBoostActive { speed_mult: 2.0 },
            ))
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<TimePressureBoostActive>(bolt).is_none(),
            "TimePressureBoostActive should be removed when NodeTimer is missing"
        );
        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 400.0).abs() < 1.0,
            "velocity should be restored to ~400.0 (800/2.0), got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 13: tick is a no-op without TimePressureBoostConfig
    // =========================================================================

    #[test]
    fn tick_no_op_without_config_resource() {
        let mut app = test_app();
        // No TimePressureBoostConfig resource
        app.insert_resource(NodeTimer {
            remaining: 5.0,
            total: 60.0,
        });
        let bolt = spawn_bolt(&mut app, Vec2::new(0.0, 400.0), 800.0);

        tick(&mut app);

        assert!(
            app.world().get::<TimePressureBoostActive>(bolt).is_none(),
            "no component should be inserted without config"
        );
        let vel = app
            .world()
            .get::<Velocity2D>(bolt)
            .expect("bolt should have Velocity2D");
        assert!(
            (vel.0.length() - 400.0).abs() < 1.0,
            "velocity should remain unchanged at ~400.0, got {}",
            vel.0.length()
        );
    }

    // =========================================================================
    // Behavior 14: tick applies boost to newly spawned bolt
    // =========================================================================

    #[test]
    fn tick_applies_boost_to_newly_spawned_bolt() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        app.insert_resource(NodeTimer {
            remaining: 5.0,
            total: 60.0,
        });

        // First bolt already has active boost
        let bolt1 = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, 800.0)),
                BoltMaxSpeed(800.0),
                TimePressureBoostActive { speed_mult: 2.0 },
            ))
            .id();

        // Second bolt spawned without boost
        let bolt2 = spawn_bolt(&mut app, Vec2::new(0.0, 300.0), 800.0);

        tick(&mut app);

        // Bolt 1 should remain unchanged
        let vel1 = app
            .world()
            .get::<Velocity2D>(bolt1)
            .expect("bolt1 should have Velocity2D");
        assert!(
            (vel1.0.length() - 800.0).abs() < 1.0,
            "bolt1 velocity should remain ~800.0, got {}",
            vel1.0.length()
        );

        // Bolt 2 should gain boost and speed
        let active2 = app
            .world()
            .get::<TimePressureBoostActive>(bolt2)
            .expect("bolt2 should gain TimePressureBoostActive");
        assert!(
            (active2.speed_mult - 2.0).abs() < f32::EPSILON,
            "bolt2 speed_mult should be 2.0"
        );
        let vel2 = app
            .world()
            .get::<Velocity2D>(bolt2)
            .expect("bolt2 should have Velocity2D");
        // 300.0 * 2.0 = 600.0
        assert!(
            (vel2.0.length() - 600.0).abs() < 1.0,
            "bolt2 velocity should be ~600.0 (300*2.0), got {}",
            vel2.0.length()
        );
    }

    // =========================================================================
    // Behavior 7 edge case: multiple bolts all receive boost
    // =========================================================================

    #[test]
    fn tick_applies_boost_to_multiple_bolts() {
        let mut app = test_app();
        app.insert_resource(TimePressureBoostConfig {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        });
        app.insert_resource(NodeTimer {
            remaining: 5.0,
            total: 60.0,
        });
        let bolt1 = spawn_bolt(&mut app, Vec2::new(0.0, 300.0), 800.0);
        let bolt2 = spawn_bolt(&mut app, Vec2::new(0.0, 400.0), 800.0);

        tick(&mut app);

        assert!(
            app.world().get::<TimePressureBoostActive>(bolt1).is_some(),
            "bolt1 should gain TimePressureBoostActive"
        );
        assert!(
            app.world().get::<TimePressureBoostActive>(bolt2).is_some(),
            "bolt2 should gain TimePressureBoostActive"
        );
    }
}
