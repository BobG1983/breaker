//! Store authoritative positions after physics runs.

use bevy::prelude::*;

use crate::interpolate::components::{InterpolateTransform, PhysicsTranslation};

/// Captures the authoritative position after physics has run.
///
/// Runs in `FixedPostUpdate` — after all `FixedUpdate` systems complete.
/// Stores the new `transform.translation` into `physics.current`.
pub(crate) fn store_authoritative(
    mut query: Query<(&Transform, &mut PhysicsTranslation), With<InterpolateTransform>>,
) {
    for (transform, mut physics) in &mut query {
        physics.current = transform.translation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn move_entity(mut query: Query<&mut Transform, With<InterpolateTransform>>) {
        for mut tf in &mut query {
            tf.translation.x += 100.0;
            tf.translation.y += 50.0;
        }
    }

    #[test]
    fn current_captures_physics_output() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, move_entity)
            .add_systems(FixedPostUpdate, store_authoritative);

        app.world_mut().spawn((
            InterpolateTransform,
            PhysicsTranslation::new(Vec3::new(0.0, 0.0, 1.0)),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        let physics = app
            .world_mut()
            .query::<&PhysicsTranslation>()
            .iter(app.world())
            .next()
            .unwrap();
        assert_eq!(
            physics.current,
            Vec3::new(100.0, 50.0, 1.0),
            "current should capture post-physics position"
        );
    }
}
