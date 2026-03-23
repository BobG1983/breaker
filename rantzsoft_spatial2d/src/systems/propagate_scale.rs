//! Propagates `Scale2D` to `Transform.scale`.

use bevy::prelude::*;

use crate::{
    components::{InterpolateTransform2D, PreviousScale, Scale2D},
    propagation::ScalePropagation,
};

type PropagateScaleQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Scale2D,
        &'static PreviousScale,
        &'static mut Transform,
        Option<&'static InterpolateTransform2D>,
        Option<&'static ScalePropagation>,
        Option<&'static ChildOf>,
    ),
>;

/// Writes `Transform.scale` from `Scale2D` (z = 1.0 always).
///
/// When `InterpolateTransform2D` is present, lerps between `PreviousScale`
/// and `Scale2D` using the fixed-timestep overstep fraction.
///
/// For children with `ScalePropagation::Absolute`, counteracts the parent's
/// scale so Bevy's `TransformPropagate` produces the correct world scale.
pub fn propagate_scale(
    time: Res<Time<Fixed>>,
    mut query: PropagateScaleQuery,
    parent_scales: Query<&Scale2D>,
) {
    let alpha = time.overstep_fraction();

    for (scale, prev_scale, mut transform, interp, prop, child_of) in &mut query {
        let (mut sx, mut sy) = if interp.is_some() {
            (
                prev_scale.x + (scale.x - prev_scale.x) * alpha,
                prev_scale.y + (scale.y - prev_scale.y) * alpha,
            )
        } else {
            (scale.x, scale.y)
        };

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
    use std::time::Duration;

    use super::*;
    use crate::components::{InterpolateTransform2D, PreviousScale};

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

        app.world_mut().spawn((
            Scale2D { x: 2.0, y: 3.0 },
            PreviousScale { x: 2.0, y: 3.0 },
            Transform::default(),
        ));

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
                PreviousScale { x: 2.0, y: 2.0 },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Scale2D { x: 3.0, y: 4.0 },
                PreviousScale { x: 3.0, y: 4.0 },
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
                PreviousScale { x: 2.0, y: 2.0 },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Scale2D { x: 3.0, y: 4.0 },
                PreviousScale { x: 3.0, y: 4.0 },
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

    // ── Interpolated scale tests ──────────────────────────────

    #[test]
    fn interpolated_scale_at_half_alpha() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, propagate_scale);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousScale { x: 1.0, y: 2.0 },
            Scale2D { x: 3.0, y: 4.0 },
            Transform::default(),
        ));

        // Default fixed timestep is 1/64 second. Accumulate half of that for ~0.5 alpha.
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

        // At alpha ~0.5: lerp(1.0, 3.0, 0.5) = 2.0, lerp(2.0, 4.0, 0.5) = 3.0
        assert!(
            (tf.scale.x - 2.0).abs() < 0.5,
            "at alpha ~0.5, scale.x should be ~2.0 but got {}",
            tf.scale.x
        );
        assert!(
            (tf.scale.y - 3.0).abs() < 0.5,
            "at alpha ~0.5, scale.y should be ~3.0 but got {}",
            tf.scale.y
        );
        assert!(
            (tf.scale.z - 1.0).abs() < f32::EPSILON,
            "scale.z should always be 1.0, got {}",
            tf.scale.z
        );
    }

    #[test]
    fn alpha_zero_uses_previous_scale() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, propagate_scale);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousScale { x: 1.0, y: 1.0 },
            Scale2D { x: 5.0, y: 5.0 },
            Transform::default(),
        ));

        // No overstep accumulated => alpha ~= 0.
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        // At alpha ~0: lerp(1.0, 5.0, 0) = 1.0
        assert!(
            (tf.scale.x - 1.0).abs() < 0.5,
            "at alpha ~0, scale.x should be ~1.0 (PreviousScale) but got {}",
            tf.scale.x
        );
        assert!(
            (tf.scale.y - 1.0).abs() < 0.5,
            "at alpha ~0, scale.y should be ~1.0 (PreviousScale) but got {}",
            tf.scale.y
        );
        assert!(
            (tf.scale.z - 1.0).abs() < f32::EPSILON,
            "scale.z should always be 1.0, got {}",
            tf.scale.z
        );
    }

    #[test]
    fn high_alpha_uses_mostly_current_scale() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, propagate_scale);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousScale { x: 1.0, y: 1.0 },
            Scale2D { x: 3.0, y: 5.0 },
            Transform::default(),
        ));

        // Accumulate 1.9 timesteps. The fixed loop runs 1 tick (consuming 1.0),
        // leaving 0.9 timestep as overstep. overstep_fraction() = 0.9.
        // lerp(1.0, 3.0, 0.9) = 2.8, lerp(1.0, 5.0, 0.9) = 4.6
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep.mul_f64(1.9));
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        // At alpha ~0.9: lerp(1.0, 3.0, 0.9) = 2.8
        assert!(
            (tf.scale.x - 2.8).abs() < 0.5,
            "at alpha ~0.9, scale.x should be ~2.8 but got {}",
            tf.scale.x
        );
        // At alpha ~0.9: lerp(1.0, 5.0, 0.9) = 4.6
        assert!(
            (tf.scale.y - 4.6).abs() < 0.5,
            "at alpha ~0.9, scale.y should be ~4.6 but got {}",
            tf.scale.y
        );
        assert!(
            (tf.scale.z - 1.0).abs() < f32::EPSILON,
            "scale.z should always be 1.0, got {}",
            tf.scale.z
        );
    }

    #[test]
    fn absolute_scale_with_interpolation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_systems(
            PostUpdate,
            propagate_scale.before(bevy::transform::TransformSystems::Propagate),
        );

        let parent = app
            .world_mut()
            .spawn((
                Scale2D { x: 2.0, y: 2.0 },
                PreviousScale { x: 2.0, y: 2.0 },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                ScalePropagation::Absolute,
                InterpolateTransform2D,
                PreviousScale { x: 1.0, y: 1.0 },
                Scale2D { x: 3.0, y: 4.0 },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // Accumulate half a fixed timestep for ~0.5 alpha.
        let half_step = Duration::from_secs_f64(1.0 / 128.0);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(half_step);
        app.update();
        app.update();

        let global = app
            .world()
            .get::<GlobalTransform>(child)
            .expect("child should have GlobalTransform");

        let global_scale = global.compute_transform().scale;
        // At alpha ~0.5: lerp(1.0, 3.0, 0.5) = 2.0, lerp(1.0, 4.0, 0.5) = 2.5
        // Absolute: these should be the world-space values.
        assert!(
            (global_scale.x - 2.0).abs() < 0.5,
            "child absolute interpolated global scale.x should be ~2.0 but got {}",
            global_scale.x
        );
        assert!(
            (global_scale.y - 2.5).abs() < 0.5,
            "child absolute interpolated global scale.y should be ~2.5 but got {}",
            global_scale.y
        );
        assert!(
            (global_scale.z - 1.0).abs() < 0.1,
            "child global scale.z should be ~1.0 but got {}",
            global_scale.z
        );
    }
}
