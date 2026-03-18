//! Visual interpolation — lerp between physics positions for smooth rendering.

use bevy::prelude::*;

use crate::interpolate::components::{InterpolateTransform, PhysicsTranslation};

/// Interpolates entity positions for smooth rendering between `FixedUpdate` ticks.
///
/// Runs in `PostUpdate` (every render frame). Uses `Time<Fixed>::overstep_fraction()`
/// as the alpha value to lerp between previous and current authoritative positions.
/// Preserves the z-coordinate to avoid disrupting render ordering.
pub(crate) fn interpolate_transform(
    time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &PhysicsTranslation), With<InterpolateTransform>>,
) {
    let alpha = time.overstep_fraction();

    for (mut transform, physics) in &mut query {
        let z = transform.translation.z;
        transform.translation.x = physics.previous.x.lerp(physics.current.x, alpha);
        transform.translation.y = physics.previous.y.lerp(physics.current.y, alpha);
        transform.translation.z = z;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, interpolate_transform);
        app
    }

    #[test]
    fn interpolation_at_zero_equals_previous() {
        let mut app = test_app();
        app.world_mut().spawn((
            InterpolateTransform,
            PhysicsTranslation {
                previous: Vec3::new(0.0, 0.0, 1.0),
                current: Vec3::new(100.0, 200.0, 1.0),
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));

        // Don't accumulate any overstep — alpha should be ~0
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .unwrap();

        // At alpha ~0, should be at or very near previous
        assert!(
            (tf.translation.x - 0.0).abs() < 1.0,
            "at alpha ~0, x should be near previous (0.0), got {}",
            tf.translation.x
        );
    }

    #[test]
    fn interpolation_at_half_overstep() {
        let mut app = test_app();
        app.world_mut().spawn((
            InterpolateTransform,
            PhysicsTranslation {
                previous: Vec3::new(0.0, 0.0, 1.0),
                current: Vec3::new(100.0, 200.0, 1.0),
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));

        // Accumulate half a timestep to get alpha ~0.5
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let half = timestep / 2;
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(half);
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .unwrap();

        // At alpha ~0.5, should be roughly halfway
        assert!(
            (tf.translation.x - 50.0).abs() < 10.0,
            "at alpha ~0.5, x should be near 50.0, got {}",
            tf.translation.x
        );
        assert!(
            (tf.translation.y - 100.0).abs() < 20.0,
            "at alpha ~0.5, y should be near 100.0, got {}",
            tf.translation.y
        );
    }

    #[test]
    fn z_coordinate_preserved() {
        let mut app = test_app();
        let z_value = 5.0;
        app.world_mut().spawn((
            InterpolateTransform,
            PhysicsTranslation {
                previous: Vec3::new(0.0, 0.0, 1.0),
                current: Vec3::new(100.0, 200.0, 1.0),
            },
            Transform::from_xyz(0.0, 0.0, z_value),
        ));

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let half = timestep / 2;
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(half);
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            (tf.translation.z - z_value).abs() < f32::EPSILON,
            "z should be preserved at {z_value}, got {}",
            tf.translation.z
        );
    }

    #[test]
    fn entities_without_marker_unaffected() {
        let mut app = test_app();
        // Entity WITHOUT InterpolateTransform marker
        app.world_mut().spawn((
            PhysicsTranslation {
                previous: Vec3::new(0.0, 0.0, 1.0),
                current: Vec3::new(100.0, 200.0, 1.0),
            },
            Transform::from_xyz(42.0, 42.0, 1.0),
        ));

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let half = timestep / 2;
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(half);
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .unwrap();
        assert_eq!(
            tf.translation,
            Vec3::new(42.0, 42.0, 1.0),
            "entity without marker should not be interpolated"
        );
    }

    #[test]
    fn interpolation_at_full_step_equals_current() {
        let mut app = test_app();
        app.world_mut().spawn((
            InterpolateTransform,
            PhysicsTranslation {
                previous: Vec3::new(0.0, 0.0, 1.0),
                current: Vec3::new(100.0, 200.0, 1.0),
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));

        // Accumulate exactly one timestep — this triggers a FixedUpdate tick,
        // which resets overstep to ~0. So accumulate just under one step.
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let almost_full = timestep.saturating_sub(Duration::from_micros(1));
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(almost_full);
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .unwrap();

        // At alpha ~1.0, should be very close to current
        assert!(
            (tf.translation.x - 100.0).abs() < 1.0,
            "at alpha ~1, x should be near 100.0, got {}",
            tf.translation.x
        );
    }
}
