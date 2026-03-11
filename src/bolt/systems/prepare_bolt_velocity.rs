//! System to prepare bolt velocity each fixed tick.
//!
//! Enforces speed clamping and minimum angle. Does NOT update position —
//! the CCD system in the physics domain handles position advancement.

use bevy::prelude::*;

use crate::bolt::{components::BoltVelocity, filters::ActiveBoltFilter, resources::BoltConfig};

/// Prepares the bolt velocity for the current timestep.
///
/// Enforces speed clamping (min/max) and minimum angle from horizontal.
/// Position advancement is handled by the CCD collision system.
pub fn prepare_bolt_velocity(
    config: Res<BoltConfig>,
    mut query: Query<&mut BoltVelocity, ActiveBoltFilter>,
) {
    for mut velocity in &mut query {
        let speed = velocity.speed();
        if speed > f32::EPSILON {
            let clamped_speed = speed.clamp(config.min_speed, config.max_speed);
            if (clamped_speed - speed).abs() > f32::EPSILON {
                velocity.value = velocity.direction() * clamped_speed;
            }

            velocity.enforce_min_angle(config.min_angle_from_horizontal);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltServing};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.add_systems(Update, prepare_bolt_velocity);
        app
    }

    #[test]
    fn move_bolt_does_not_translate_position() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        app.update();

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
            ))
            .id();

        app.update();

        let vel = app.world().get::<BoltVelocity>(entity).unwrap();
        assert!(
            (vel.speed() - 1.0).abs() < f32::EPSILON,
            "serving bolt velocity should not be clamped, got speed={}",
            vel.speed()
        );
    }

    #[test]
    fn speed_below_min_is_clamped_up() {
        let mut app = test_app();
        let config = app.world().resource::<BoltConfig>().clone();

        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 1.0), // far below min_speed
        ));

        app.update();

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
