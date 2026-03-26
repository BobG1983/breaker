//! Propagates `Rotation2D` to `Transform.rotation` with interpolation.

use bevy::prelude::*;

use crate::{
    components::{InterpolateTransform2D, PreviousRotation, Rotation2D},
    propagation::RotationPropagation,
};

type PropagateRotationQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Rotation2D,
        &'static mut Transform,
        Option<&'static InterpolateTransform2D>,
        Option<&'static PreviousRotation>,
        Option<&'static RotationPropagation>,
        Option<&'static ChildOf>,
    ),
>;

/// Writes `Transform.rotation` from `Rotation2D`. When
/// `InterpolateTransform2D` is present, lerps between `PreviousRotation`
/// and `Rotation2D` using the fixed-timestep overstep fraction.
///
/// For children with `RotationPropagation::Absolute`, counteracts the parent's
/// rotation so Bevy's `TransformPropagate` produces the correct world rotation.
pub fn propagate_rotation(
    time: Res<Time<Fixed>>,
    mut query: PropagateRotationQuery,
    parent_rotations: Query<&Rotation2D>,
) {
    let alpha = time.overstep_fraction();

    for (rot, mut transform, interp, prev_rot, prop, child_of) in &mut query {
        // Interpolate or use current rotation.
        let base = if interp.is_some() {
            if let Some(prev) = prev_rot {
                prev.0.nlerp(rot.0, alpha)
            } else {
                rot.0
            }
        } else {
            rot.0
        };

        let mut angle = base.as_radians();

        // Parent/child: counteract parent's rotation for Absolute propagation.
        if let Some(child_of) = child_of
            && prop.is_some_and(|p| *p == RotationPropagation::Absolute)
            && let Ok(parent_rot) = parent_rotations.get(child_of.parent())
        {
            angle -= parent_rot.0.as_radians();
        }

        transform.rotation = Quat::from_rotation_z(angle);
    }
}

#[cfg(test)]
mod tests {
    use std::{f32::consts::FRAC_PI_2, time::Duration};

    use super::*;

    fn tick_fixed(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 15: Basic rotation to transform ──

    #[test]
    fn basic_rotation_to_transform() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_rotation);

        app.world_mut()
            .spawn((Rotation2D::from_degrees(90.0), Transform::default()));

        tick_fixed(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        let expected = Quat::from_rotation_z(FRAC_PI_2);
        let actual = tf.rotation;
        assert!(
            actual.abs_diff_eq(expected, 1e-4),
            "rotation should be ~90 degrees Z. expected {expected:?}, got {actual:?}"
        );
    }

    // ── Behavior 16: Interpolated rotation at alpha=0.5 ──

    #[test]
    fn interpolated_rotation_at_half_alpha() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, propagate_rotation);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousRotation::default(), // 0 degrees
            Rotation2D::from_degrees(90.0),
            Transform::default(),
        ));

        // Accumulate half a fixed timestep for ~0.5 alpha.
        let half_step = Duration::from_secs_f64(1.0 / 128.0);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(half_step);
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        let expected = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4); // 45 degrees
        let actual = tf.rotation;
        assert!(
            actual.abs_diff_eq(expected, 0.1),
            "at alpha ~0.5, rotation should be ~45 degrees. expected {expected:?}, got {actual:?}"
        );
    }

    // ── Behavior 17: Entity without Rotation2D not affected ──

    #[test]
    fn entity_without_rotation_not_affected() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_rotation);

        let custom_rot = Quat::from_rotation_z(1.234);
        app.world_mut().spawn(Transform::from_rotation(custom_rot));

        tick_fixed(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert!(
            tf.rotation.abs_diff_eq(custom_rot, 1e-6),
            "rotation should be unchanged without Rotation2D component"
        );
    }

    // ── Behavior 18: Parent/child Relative rotation ──

    #[test]
    fn parent_child_relative_rotation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_systems(
            FixedUpdate,
            propagate_rotation.before(bevy::transform::TransformSystems::Propagate),
        );

        let parent = app
            .world_mut()
            .spawn((
                Rotation2D::from_degrees(90.0),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Rotation2D::from_degrees(45.0),
                RotationPropagation::Relative,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        tick_fixed(&mut app);
        tick_fixed(&mut app);

        let global = app
            .world()
            .get::<GlobalTransform>(child)
            .expect("child should have GlobalTransform");

        // Parent 90 + child 45 = 135 degrees total.
        let expected = Quat::from_rotation_z(135.0_f32.to_radians());
        let actual = global.compute_transform().rotation;
        assert!(
            actual.abs_diff_eq(expected, 0.1),
            "child global rotation should be ~135 degrees. expected {expected:?}, got {actual:?}"
        );
    }

    // ── Behavior 19: Parent/child Absolute rotation ──

    #[test]
    fn parent_child_absolute_rotation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_systems(
            FixedUpdate,
            propagate_rotation.before(bevy::transform::TransformSystems::Propagate),
        );

        let parent = app
            .world_mut()
            .spawn((
                Rotation2D::from_degrees(90.0),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Rotation2D::from_degrees(45.0),
                RotationPropagation::Absolute,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        tick_fixed(&mut app);
        tick_fixed(&mut app);

        let global = app
            .world()
            .get::<GlobalTransform>(child)
            .expect("child should have GlobalTransform");

        // Absolute: child's world rotation should be exactly 45 degrees,
        // ignoring the parent's 90 degrees.
        let expected = Quat::from_rotation_z(45.0_f32.to_radians());
        let actual = global.compute_transform().rotation;
        assert!(
            actual.abs_diff_eq(expected, 0.1),
            "child global rotation should be absolute 45 degrees. expected {expected:?}, got {actual:?}"
        );
    }
}
