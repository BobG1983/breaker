//! Propagates `Scale2D` to `Transform.scale`.

use bevy::prelude::*;

use crate::{components::Scale2D, propagation::ScalePropagation};

/// Writes `Transform.scale` from `Scale2D` (z = 1.0 always).
///
/// For children with `ScalePropagation::Absolute`, counteracts the parent's
/// scale so Bevy's `TransformPropagate` produces the correct world scale.
pub fn propagate_scale(
    mut query: Query<(
        &Scale2D,
        &mut Transform,
        Option<&ScalePropagation>,
        Option<&ChildOf>,
    )>,
    parent_scales: Query<&Scale2D>,
) {
    for (scale, mut transform, prop, child_of) in &mut query {
        let mut sx = scale.x;
        let mut sy = scale.y;

        // Parent/child: counteract parent's scale for Absolute propagation.
        if let Some(child_of) = child_of
            && prop.is_some_and(|p| *p == ScalePropagation::Absolute)
            && let Ok(parent_scale) = parent_scales.get(child_of.parent())
        {
            sx /= parent_scale.x;
            sy /= parent_scale.y;
        }

        transform.scale = Vec3::new(sx, sy, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick_fixed(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 20: Basic scale to transform ──

    #[test]
    fn basic_scale_to_transform() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_scale);

        app.world_mut()
            .spawn((Scale2D { x: 2.0, y: 3.0 }, Transform::default()));

        tick_fixed(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert_eq!(
            tf.scale,
            Vec3::new(2.0, 3.0, 1.0),
            "Transform.scale should be (2, 3, 1)"
        );
    }

    // ── Behavior 21: Entity without Scale2D not affected ──

    #[test]
    fn entity_without_scale_not_affected() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_scale);

        app.world_mut()
            .spawn(Transform::from_scale(Vec3::new(5.0, 5.0, 5.0)));

        tick_fixed(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert_eq!(
            tf.scale,
            Vec3::new(5.0, 5.0, 5.0),
            "Transform.scale should be unchanged without Scale2D"
        );
    }

    // ── Behavior 22: Parent/child Relative scale ──

    #[test]
    fn parent_child_relative_scale() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_systems(
            FixedUpdate,
            propagate_scale.before(bevy::transform::TransformSystems::Propagate),
        );

        let parent = app
            .world_mut()
            .spawn((
                Scale2D { x: 2.0, y: 2.0 },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Scale2D { x: 3.0, y: 4.0 },
                ScalePropagation::Relative,
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

        let global_scale = global.compute_transform().scale;
        // Relative: child local (3, 4, 1) * parent (2, 2, 1) = (6, 8, 1).
        assert!(
            (global_scale.x - 6.0).abs() < 0.1
                && (global_scale.y - 8.0).abs() < 0.1
                && (global_scale.z - 1.0).abs() < 0.1,
            "child global scale should be (6, 8, 1) but got {global_scale:?}"
        );
    }

    // ── Behavior 23: Parent/child Absolute scale ──

    #[test]
    fn parent_child_absolute_scale() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_systems(
            FixedUpdate,
            propagate_scale.before(bevy::transform::TransformSystems::Propagate),
        );

        let parent = app
            .world_mut()
            .spawn((
                Scale2D { x: 2.0, y: 2.0 },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Scale2D { x: 3.0, y: 4.0 },
                ScalePropagation::Absolute,
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

        let global_scale = global.compute_transform().scale;
        // Absolute: child's world scale should be (3, 4, 1), ignoring parent.
        // Counteract: local = (3/2, 4/2, 1) = (1.5, 2, 1). Bevy * parent (2, 2, 1) = (3, 4, 1).
        assert!(
            (global_scale.x - 3.0).abs() < 0.1
                && (global_scale.y - 4.0).abs() < 0.1
                && (global_scale.z - 1.0).abs() < 0.1,
            "child global scale should be absolute (3, 4, 1) but got {global_scale:?}"
        );
    }
}
