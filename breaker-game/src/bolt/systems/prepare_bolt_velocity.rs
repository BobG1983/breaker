//! System to prepare bolt velocity each fixed tick.
//!
//! Enforces speed clamping and minimum angle. Does NOT update position —
//! the CCD system in the physics domain handles position advancement.

use bevy::prelude::*;

use crate::{
    bolt::{components::*, filters::ActiveFilter},
    breaker::components::{Breaker, MinAngleFromHorizontal},
    chips::components::BoltSpeedBoost,
};

/// Prepares the bolt velocity for the current timestep.
///
/// Enforces speed clamping (min/max) and minimum angle from horizontal.
/// Position advancement is handled by the CCD collision system.
pub(crate) fn prepare_bolt_velocity(
    mut query: Query<
        (
            &mut BoltVelocity,
            &BoltMinSpeed,
            &BoltMaxSpeed,
            Option<&BoltSpeedBoost>,
        ),
        ActiveFilter,
    >,
    breaker_query: Query<&MinAngleFromHorizontal, (With<Breaker>, Without<Bolt>)>,
) {
    let Ok(min_angle) = breaker_query.single() else {
        return;
    };

    for (mut velocity, min_speed, max_speed, speed_boost) in &mut query {
        let speed = velocity.speed();
        if speed > f32::EPSILON {
            let boost = speed_boost.map_or(0.0, |b| b.0);
            let effective_min = min_speed.0 + boost;
            let effective_max = max_speed.0 + boost;
            let clamped_speed = speed.clamp(effective_min, effective_max);
            if (clamped_speed - speed).abs() > f32::EPSILON {
                velocity.value = velocity.direction() * clamped_speed;
            }

            velocity.enforce_min_angle(min_angle.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::{
            components::{Bolt, BoltBaseSpeed, BoltServing},
            resources::BoltConfig,
        },
        breaker::resources::BreakerConfig,
        chips::components::BoltSpeedBoost,
    };

    fn bolt_param_bundle() -> (BoltBaseSpeed, BoltMinSpeed, BoltMaxSpeed) {
        let bolt_config = BoltConfig::default();
        (
            BoltBaseSpeed(bolt_config.base_speed),
            BoltMinSpeed(bolt_config.min_speed),
            BoltMaxSpeed(bolt_config.max_speed),
        )
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, prepare_bolt_velocity);
        // Spawn breaker with MinAngleFromHorizontal for the system to read
        let breaker_config = BreakerConfig::default();
        app.world_mut().spawn((
            Breaker,
            MinAngleFromHorizontal(breaker_config.min_angle_from_horizontal.to_radians()),
        ));
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
    fn move_bolt_does_not_translate_position() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            bolt_param_bundle(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        tick(&mut app);

        let tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");

        assert!(
            tf.translation.y.abs() < f32::EPSILON,
            "move_bolt should NOT update position (CCD handles that), got y={}",
            tf.translation.y
        );
    }

    #[test]
    fn serving_bolt_velocity_unchanged() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                BoltVelocity::new(0.0, 1.0), // below min_speed
                bolt_param_bundle(),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            (vel.speed() - 1.0).abs() < f32::EPSILON,
            "serving bolt velocity should not be clamped, got speed={}",
            vel.speed()
        );
    }

    #[test]
    fn no_breaker_leaves_velocity_unchanged() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, prepare_bolt_velocity);
        // No breaker entity spawned

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, 1.0), // below min, but no breaker → early return
                bolt_param_bundle(),
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            (vel.speed() - 1.0).abs() < f32::EPSILON,
            "without breaker, velocity should be unchanged, got speed={}",
            vel.speed()
        );
    }

    #[test]
    fn speed_below_min_is_clamped_up() {
        let mut app = test_app();
        let config = BoltConfig::default();

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 1.0), // far below min_speed
            bolt_param_bundle(),
        ));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(
            vel.speed() >= config.min_speed - f32::EPSILON,
            "speed {} should be at least min_speed {}",
            vel.speed(),
            config.min_speed
        );
    }

    // --- BoltSpeedBoost tests ---

    /// [`BoltSpeedBoost`] raises the effective minimum speed.
    ///
    /// Given: speed=100, min=200, max=600, boost=100 → `effective_min`=300.
    /// Speed 100 < 300 → should clamp UP to 300.
    /// RED: system ignores `_speed_boost`, clamps to base min (200), not 300 → FAIL.
    #[test]
    fn bolt_speed_boost_raises_effective_min_speed() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, 100.0), // speed=100
                BoltMinSpeed(200.0),
                BoltMaxSpeed(600.0),
                BoltSpeedBoost(100.0), // effective_min = 300
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            vel.speed() >= 300.0 - f32::EPSILON,
            "speed {} should be at least effective_min 300.0 (base 200 + boost 100)",
            vel.speed()
        );
    }

    /// [`BoltSpeedBoost`] raises the effective maximum speed.
    ///
    /// Given: speed=800, min=200, max=600, boost=100 → `effective_max`=700.
    /// Speed 800 > 700 → should clamp DOWN to 700.
    /// RED: system ignores `_speed_boost`, clamps to base max (600), not 700 → FAIL.
    #[test]
    fn bolt_speed_boost_raises_effective_max_speed() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, 800.0), // speed=800
                BoltMinSpeed(200.0),
                BoltMaxSpeed(600.0),
                BoltSpeedBoost(100.0), // effective_max = 700
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            vel.speed() >= 700.0 - f32::EPSILON,
            "speed {} should be at least effective_max 700.0 (base 600 + boost 100), not clamped to base 600",
            vel.speed()
        );
    }

    /// Regression guard: without [`BoltSpeedBoost`], base clamping is unchanged.
    ///
    /// Given: speed=100, min=200, max=600, NO boost.
    /// Speed 100 < 200 → clamped to 200 (base min). No boost applied.
    /// GREEN: this should pass with the current stub implementation.
    #[test]
    fn no_bolt_speed_boost_uses_base_min_speed() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, 100.0), // speed=100
                BoltMinSpeed(200.0),
                BoltMaxSpeed(600.0),
                // No BoltSpeedBoost
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            (vel.speed() - 200.0).abs() < 1.0,
            "speed {} should be clamped to base min 200.0 when no boost present",
            vel.speed()
        );
    }

    /// [`BoltServing`] bolt is not affected by [`BoltSpeedBoost`] (excluded by `ActiveFilter`).
    ///
    /// Given: serving bolt, speed=1, min=200, max=600, boost=100.
    /// `ActiveFilter` excludes [`BoltServing`] → velocity unchanged at speed=1.
    /// GREEN: this should pass because the `ActiveFilter` already excludes serving bolts.
    #[test]
    fn serving_bolt_not_affected_by_bolt_speed_boost() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                BoltVelocity::new(0.0, 1.0), // speed=1, below any min
                BoltMinSpeed(200.0),
                BoltMaxSpeed(600.0),
                BoltSpeedBoost(100.0), // effective_min would be 300
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            (vel.speed() - 1.0).abs() < f32::EPSILON,
            "serving bolt speed {} should be unchanged at 1.0 (excluded by ActiveFilter)",
            vel.speed()
        );
    }

    /// `BoltSpeedBoost(0.0)` is identical to no boost — base clamping applies.
    ///
    /// Given: speed=600, min=200, max=600, boost=0.0 → `effective_max`=600.
    /// Speed 600 == `effective_max` → should remain at 600 (no change needed).
    /// GREEN: this should pass because boost of 0 means no change to the clamp range.
    #[test]
    fn bolt_speed_boost_zero_same_as_no_boost() {
        let mut app = test_app();

        let entity = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, 600.0), // speed=600, exactly at base max
                BoltMinSpeed(200.0),
                BoltMaxSpeed(600.0),
                BoltSpeedBoost(0.0), // zero boost — no change expected
            ))
            .id();

        tick(&mut app);

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            (vel.speed() - 600.0).abs() < 1.0,
            "speed {} should remain at 600.0 when boost is 0.0 (at base max, no effective change)",
            vel.speed()
        );
    }
}
