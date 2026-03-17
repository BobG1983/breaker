//! System to prepare bolt velocity each fixed tick.
//!
//! Enforces speed clamping and minimum angle. Does NOT update position —
//! the CCD system in the physics domain handles position advancement.

use bevy::prelude::*;

use crate::{
    bolt::{components::*, filters::ActiveBoltFilter},
    breaker::components::{Breaker, MinAngleFromHorizontal},
};

/// Prepares the bolt velocity for the current timestep.
///
/// Enforces speed clamping (min/max) and minimum angle from horizontal.
/// Position advancement is handled by the CCD collision system.
pub fn prepare_bolt_velocity(
    mut query: Query<(&mut BoltVelocity, &BoltMinSpeed, &BoltMaxSpeed), ActiveBoltFilter>,
    breaker_query: Query<&MinAngleFromHorizontal, (With<Breaker>, Without<Bolt>)>,
) {
    let Ok(min_angle) = breaker_query.single() else {
        return;
    };

    for (mut velocity, min_speed, max_speed) in &mut query {
        let speed = velocity.speed();
        if speed > f32::EPSILON {
            let clamped_speed = speed.clamp(min_speed.0, max_speed.0);
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
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, prepare_bolt_velocity);
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
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, prepare_bolt_velocity);
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
}
