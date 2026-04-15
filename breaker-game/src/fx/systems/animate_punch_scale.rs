//! Animate `PunchScale` — ticks scale overshoot back to 1.0 then removes the component.

use bevy::prelude::*;

use crate::fx::PunchScale;

/// Ticks `PunchScale` timers and animates `Transform.scale` from overshoot
/// back to 1.0. Removes the component (does NOT despawn the entity) when done.
pub(crate) fn animate_punch_scale(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PunchScale, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut punch, mut transform) in &mut query {
        punch.timer -= dt;
        if punch.timer <= 0.0 || punch.duration <= 0.0 {
            transform.scale = Vec3::splat(1.0);
            commands.entity(entity).remove::<PunchScale>();
            continue;
        }
        let t = punch.timer / punch.duration;
        transform.scale = Vec3::splat((punch.overshoot - 1.0).mul_add(t, 1.0));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::prelude::*;

    /// Build a test app that advances time by `dt` each update.
    fn test_app(dt: Duration) -> App {
        TestAppBuilder::new()
            .insert_resource(TimeUpdateStrategy::ManualDuration(dt))
            .with_system(Update, animate_punch_scale)
            .build()
    }

    /// Default 16ms timestep for tests that just need time to advance.
    fn default_app() -> App {
        test_app(Duration::from_millis(16))
    }

    #[test]
    fn punch_scale_animates_scale_from_overshoot_toward_one() {
        // dt=50ms, duration=100ms → halfway through animation
        let mut app = test_app(Duration::from_millis(50));

        let entity = app
            .world_mut()
            .spawn((
                PunchScale {
                    timer:     0.1,
                    duration:  0.1,
                    overshoot: 1.15,
                },
                Transform::from_scale(Vec3::splat(1.15)),
            ))
            .id();

        // First update initializes time, second advances it
        app.update();
        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        let scale_x = transform.scale.x;
        let scale_y = transform.scale.y;

        // At halfway through the animation, scale should be between 1.0 and 1.15
        assert!(
            scale_x > 1.0 && scale_x < 1.15,
            "scale.x should be between 1.0 and 1.15 at midpoint, got {scale_x}"
        );
        assert!(
            scale_y > 1.0 && scale_y < 1.15,
            "scale.y should be between 1.0 and 1.15 at midpoint, got {scale_y}"
        );

        // Timer should have decremented
        let punch = app.world().get::<PunchScale>(entity).unwrap();
        assert!(
            (punch.timer - 0.05).abs() < 0.01,
            "timer should be approximately 0.05, got {}",
            punch.timer
        );
    }

    #[test]
    fn punch_scale_with_overshoot_one_keeps_scale_at_one() {
        let mut app = test_app(Duration::from_millis(50));

        let entity = app
            .world_mut()
            .spawn((
                PunchScale {
                    timer:     0.1,
                    duration:  0.1,
                    overshoot: 1.0,
                },
                Transform::default(),
            ))
            .id();

        app.update();
        app.update();

        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!(
            (transform.scale.x - 1.0).abs() < f32::EPSILON,
            "scale.x should stay at 1.0 when overshoot is 1.0, got {}",
            transform.scale.x
        );
        assert!(
            (transform.scale.y - 1.0).abs() < f32::EPSILON,
            "scale.y should stay at 1.0 when overshoot is 1.0, got {}",
            transform.scale.y
        );

        // Timer must still be ticking even when overshoot is 1.0
        let punch = app.world().get::<PunchScale>(entity).unwrap();
        assert!(
            (punch.timer - 0.05).abs() < 0.01,
            "timer should decrement to ~0.05 even with overshoot 1.0, got {}",
            punch.timer
        );
    }

    #[test]
    fn punch_scale_removes_component_when_expired_entity_survives() {
        let mut app = default_app();

        let entity = app
            .world_mut()
            .spawn((
                PunchScale {
                    timer:     0.0,
                    duration:  0.1,
                    overshoot: 1.15,
                },
                Transform::from_scale(Vec3::splat(1.15)),
            ))
            .id();

        // First update: timer is already 0 → should complete
        app.update();

        // Entity must still exist
        assert!(
            app.world().get_entity(entity).is_ok(),
            "entity should NOT be despawned when PunchScale completes"
        );

        // PunchScale component must be removed
        assert!(
            app.world().get::<PunchScale>(entity).is_none(),
            "PunchScale component should be removed when timer expires"
        );

        // Scale should be reset to 1.0
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!(
            (transform.scale.x - 1.0).abs() < f32::EPSILON,
            "scale.x should be 1.0 after PunchScale completes, got {}",
            transform.scale.x
        );
        assert!(
            (transform.scale.y - 1.0).abs() < f32::EPSILON,
            "scale.y should be 1.0 after PunchScale completes, got {}",
            transform.scale.y
        );
    }

    #[test]
    fn punch_scale_timer_crosses_zero_mid_tick() {
        let mut app = default_app();

        let entity = app
            .world_mut()
            .spawn((
                PunchScale {
                    timer:     0.001,
                    duration:  0.1,
                    overshoot: 1.15,
                },
                Transform::from_scale(Vec3::splat(1.15)),
            ))
            .id();

        // Two updates: first initializes time, second advances dt(16ms) past the timer(1ms)
        app.update();
        app.update();

        // Entity must still exist
        assert!(
            app.world().get_entity(entity).is_ok(),
            "entity should NOT be despawned"
        );

        // PunchScale removed
        assert!(
            app.world().get::<PunchScale>(entity).is_none(),
            "PunchScale should be removed when timer crosses zero mid-tick"
        );

        // Scale reset to 1.0
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!(
            (transform.scale.x - 1.0).abs() < f32::EPSILON,
            "scale.x should be 1.0 after completion, got {}",
            transform.scale.x
        );
        assert!(
            (transform.scale.y - 1.0).abs() < f32::EPSILON,
            "scale.y should be 1.0 after completion, got {}",
            transform.scale.y
        );
    }

    #[test]
    fn punch_scale_duration_zero_does_not_divide_by_zero() {
        let mut app = default_app();

        let entity = app
            .world_mut()
            .spawn((
                PunchScale {
                    timer:     0.0,
                    duration:  0.0,
                    overshoot: 1.15,
                },
                Transform::from_scale(Vec3::splat(1.15)),
            ))
            .id();

        // Should not panic from division by zero
        app.update();

        // Entity survives
        assert!(
            app.world().get_entity(entity).is_ok(),
            "entity should NOT be despawned"
        );

        // PunchScale removed
        assert!(
            app.world().get::<PunchScale>(entity).is_none(),
            "PunchScale should be removed when duration is 0.0"
        );

        // Scale reset to 1.0
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert!(
            (transform.scale.x - 1.0).abs() < f32::EPSILON,
            "scale.x should be 1.0 after zero-duration PunchScale, got {}",
            transform.scale.x
        );
        assert!(
            (transform.scale.y - 1.0).abs() < f32::EPSILON,
            "scale.y should be 1.0 after zero-duration PunchScale, got {}",
            transform.scale.y
        );
    }

    #[test]
    fn entity_without_transform_is_unaffected() {
        let mut app = default_app();

        // Entity with PunchScale but no Transform — not matched by the query
        let entity = app
            .world_mut()
            .spawn(PunchScale {
                timer:     0.1,
                duration:  0.1,
                overshoot: 1.15,
            })
            .id();

        app.update();

        // Entity should still exist
        assert!(
            app.world().get_entity(entity).is_ok(),
            "entity without Transform should not be affected"
        );

        // PunchScale should still be present (not matched by query, so untouched)
        let punch = app.world().get::<PunchScale>(entity).unwrap();
        assert!(
            (punch.timer - 0.1).abs() < f32::EPSILON,
            "timer should be unchanged for entity without Transform, got {}",
            punch.timer
        );
    }

    #[test]
    fn multiple_punch_scale_entities_tick_independently() {
        let mut app = test_app(Duration::from_millis(50));

        // Entity A: timer 0.05, will expire this tick (dt=0.05 >= timer)
        let entity_a = app
            .world_mut()
            .spawn((
                PunchScale {
                    timer:     0.05,
                    duration:  0.1,
                    overshoot: 1.15,
                },
                Transform::from_scale(Vec3::splat(1.15)),
            ))
            .id();

        // Entity B: timer 0.2, should still be animating after one tick
        let entity_b = app
            .world_mut()
            .spawn((
                PunchScale {
                    timer:     0.2,
                    duration:  0.2,
                    overshoot: 1.3,
                },
                Transform::from_scale(Vec3::splat(1.3)),
            ))
            .id();

        // First update initializes time, second advances it by 50ms
        app.update();
        app.update();

        // Entity A: timer expired → PunchScale removed
        assert!(
            app.world().get::<PunchScale>(entity_a).is_none(),
            "entity A PunchScale should be removed (timer expired)"
        );
        // Entity A still exists
        assert!(
            app.world().get_entity(entity_a).is_ok(),
            "entity A should still exist after PunchScale removal"
        );

        // Entity B: still animating, PunchScale present
        let punch_b = app.world().get::<PunchScale>(entity_b);
        assert!(
            punch_b.is_some(),
            "entity B PunchScale should still be present (timer not expired)"
        );
        let punch_b = punch_b.unwrap();
        assert!(
            (punch_b.timer - 0.15).abs() < 0.01,
            "entity B timer should be approximately 0.15, got {}",
            punch_b.timer
        );
    }
}
