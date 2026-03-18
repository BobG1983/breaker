//! Restore authoritative positions before physics runs.

use bevy::prelude::*;

use crate::interpolate::components::{InterpolateTransform, PhysicsTranslation};

/// Restores the authoritative (physics) position before the `FixedUpdate` tick.
///
/// Runs in `FixedFirst`:
/// - Saves `current` into `previous` (shift the history)
/// - Sets `transform.translation` back to `current` (undo render interpolation)
///
/// This ensures physics systems always see the true authoritative position,
/// not the interpolated visual position from the previous render frame.
pub(crate) fn restore_authoritative(
    mut query: Query<(&mut Transform, &mut PhysicsTranslation), With<InterpolateTransform>>,
) {
    for (mut transform, mut physics) in &mut query {
        physics.previous = physics.current;
        transform.translation = physics.current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedFirst, restore_authoritative);
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
    fn previous_updated_to_current() {
        let mut app = test_app();
        app.world_mut().spawn((
            InterpolateTransform,
            PhysicsTranslation {
                previous: Vec3::new(0.0, 0.0, 1.0),
                current: Vec3::new(10.0, 20.0, 1.0),
            },
            Transform::from_xyz(5.0, 10.0, 1.0), // interpolated position
        ));
        tick(&mut app);

        let physics = app
            .world_mut()
            .query::<&PhysicsTranslation>()
            .iter(app.world())
            .next()
            .unwrap();
        assert_eq!(
            physics.previous,
            Vec3::new(10.0, 20.0, 1.0),
            "previous should be updated to old current"
        );
    }

    #[test]
    fn transform_restored_to_current() {
        let mut app = test_app();
        app.world_mut().spawn((
            InterpolateTransform,
            PhysicsTranslation {
                previous: Vec3::new(0.0, 0.0, 1.0),
                current: Vec3::new(10.0, 20.0, 1.0),
            },
            Transform::from_xyz(5.0, 10.0, 1.0), // interpolated position
        ));
        tick(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .unwrap();
        assert_eq!(
            tf.translation,
            Vec3::new(10.0, 20.0, 1.0),
            "transform should be restored to authoritative current"
        );
    }

    #[test]
    fn entities_without_marker_unaffected() {
        let mut app = test_app();
        // Entity WITHOUT InterpolateTransform
        app.world_mut().spawn((
            PhysicsTranslation {
                previous: Vec3::ZERO,
                current: Vec3::new(10.0, 20.0, 1.0),
            },
            Transform::from_xyz(99.0, 99.0, 1.0),
        ));
        tick(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .unwrap();
        assert_eq!(
            tf.translation,
            Vec3::new(99.0, 99.0, 1.0),
            "entity without marker should not be affected"
        );
    }
}
