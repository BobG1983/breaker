//! Derives `Transform` from `GlobalPosition2D`, `GlobalRotation2D`,
//! `GlobalScale2D` with interpolation and visual offset. Replaces the
//! old `propagate_position`, `propagate_rotation`, `propagate_scale` systems.
//!
//! For child entities, counteracts the parent's global transform so that
//! Bevy's built-in `TransformPropagate` (which computes
//! `child.GlobalTransform = parent.GlobalTransform * child.Transform`)
//! produces the correct world-space result.

use bevy::prelude::*;

use crate::{
    components::{
        GlobalPosition2D, GlobalRotation2D, GlobalScale2D, InterpolateTransform2D,
        PreviousPosition, PreviousRotation, PreviousScale, VisualOffset,
    },
    draw_layer::DrawLayer,
};

/// Query type for `derive_transform` — avoids clippy `type_complexity`.
type DeriveTransformQuery<'w, 's, D> = Query<
    'w,
    's,
    (
        &'static GlobalPosition2D,
        &'static GlobalRotation2D,
        &'static GlobalScale2D,
        &'static D,
        &'static mut Transform,
        Option<&'static InterpolateTransform2D>,
        Option<&'static PreviousPosition>,
        Option<&'static PreviousRotation>,
        Option<&'static PreviousScale>,
        Option<&'static VisualOffset>,
        Option<&'static ChildOf>,
    ),
>;

/// Query type for reading parent globals during counteraction.
type ParentGlobalsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static GlobalPosition2D,
        &'static GlobalRotation2D,
        &'static GlobalScale2D,
    ),
>;

/// Writes `Transform` from `GlobalPosition2D`, `GlobalRotation2D`,
/// `GlobalScale2D`, with optional interpolation via `Previous*` components
/// and `VisualOffset`. Only processes entities that have a `DrawLayer`.
///
/// For child entities, subtracts the parent's global position, rotation, and
/// scale so that Bevy's `TransformPropagate` adds them back correctly.
pub fn derive_transform<D: DrawLayer>(
    time: Res<Time<Fixed>>,
    mut query: DeriveTransformQuery<D>,
    parent_query: ParentGlobalsQuery,
) {
    let alpha = time.overstep_fraction();

    for (
        g_pos,
        g_rot,
        g_scale,
        layer,
        mut transform,
        interp,
        prev_pos,
        prev_rot,
        prev_scale,
        offset,
        child_of,
    ) in &mut query
    {
        // Position: interpolate if markers present, then extend to Vec3 with z.
        let base_pos = if interp.is_some() {
            if let Some(prev) = prev_pos {
                prev.0.lerp(g_pos.0, alpha)
            } else {
                g_pos.0
            }
        } else {
            g_pos.0
        };
        let mut translation = base_pos.extend(layer.z());

        // Rotation: interpolate if markers present.
        let base_rot = if interp.is_some() {
            if let Some(prev) = prev_rot {
                prev.0.nlerp(g_rot.0, alpha)
            } else {
                g_rot.0
            }
        } else {
            g_rot.0
        };
        let mut rotation_radians = base_rot.as_radians();

        // Scale: interpolate if markers present.
        let (mut sx, mut sy) = if interp.is_some() {
            if let Some(prev) = prev_scale {
                (
                    prev.x + (g_scale.x - prev.x) * alpha,
                    prev.y + (g_scale.y - prev.y) * alpha,
                )
            } else {
                (g_scale.x, g_scale.y)
            }
        } else {
            (g_scale.x, g_scale.y)
        };

        // Counteract parent globals for children so TransformPropagate
        // produces the correct world-space result.
        if let Some(child_of) = child_of
            && let Ok((parent_pos, parent_rot, parent_scale)) = parent_query.get(child_of.parent())
        {
            translation.x -= parent_pos.0.x;
            translation.y -= parent_pos.0.y;

            rotation_radians -= parent_rot.0.as_radians();

            if parent_scale.x.abs() > f32::EPSILON {
                sx /= parent_scale.x;
            }
            if parent_scale.y.abs() > f32::EPSILON {
                sy /= parent_scale.y;
            }
        }

        // Apply visual offset.
        if let Some(offset) = offset {
            translation += offset.0;
        }
        transform.translation = translation;

        transform.rotation = Quat::from_rotation_z(rotation_radians);

        transform.scale = Vec3::new(sx, sy, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[derive(Component, Clone, Copy, Debug, Default, Reflect)]
    enum TestDrawLayer {
        #[default]
        A,
        B,
    }

    impl DrawLayer for TestDrawLayer {
        fn z(&self) -> f32 {
            match self {
                Self::A => 0.0,
                Self::B => 1.0,
            }
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 22: Non-interpolated Transform from Global* ──

    #[test]
    fn non_interpolated_transform_from_globals() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, derive_transform::<TestDrawLayer>);

        let entity = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(10.0, 20.0)),
                GlobalRotation2D(Rot2::degrees(45.0)),
                GlobalScale2D { x: 2.0, y: 3.0 },
                TestDrawLayer::B,
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();

        let expected_pi_4 = std::f32::consts::FRAC_PI_4;
        assert_eq!(
            tf.translation,
            Vec3::new(10.0, 20.0, 1.0),
            "translation should be (10, 20, z=1.0 from TestDrawLayer::B)"
        );
        let expected_rot = Quat::from_rotation_z(expected_pi_4);
        assert!(
            tf.rotation.abs_diff_eq(expected_rot, 1e-4),
            "rotation should be ~45 degrees Z. expected {expected_rot:?}, got {:?}",
            tf.rotation
        );
        assert_eq!(
            tf.scale,
            Vec3::new(2.0, 3.0, 1.0),
            "scale should be (2, 3, 1)"
        );
    }

    // ── Behavior 23: Interpolated at alpha ~0.5 ──

    #[test]
    fn interpolated_at_half_alpha_lerps_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, derive_transform::<TestDrawLayer>);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousPosition(Vec2::new(0.0, 0.0)),
            GlobalPosition2D(Vec2::new(10.0, 0.0)),
            GlobalRotation2D::default(),
            GlobalScale2D::default(),
            PreviousRotation::default(),
            PreviousScale::default(),
            TestDrawLayer::A,
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

        assert!(
            (tf.translation.x - 5.0).abs() < 0.5,
            "at alpha ~0.5, x should be ~5.0 but got {}",
            tf.translation.x
        );
    }

    // ── Behavior 24: Interpolated at alpha ~0 uses Previous* ──

    #[test]
    fn interpolated_at_alpha_zero_uses_previous() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, derive_transform::<TestDrawLayer>);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousPosition(Vec2::new(100.0, 0.0)),
            GlobalPosition2D(Vec2::new(200.0, 0.0)),
            GlobalRotation2D::default(),
            GlobalScale2D::default(),
            PreviousRotation::default(),
            PreviousScale::default(),
            TestDrawLayer::A,
            Transform::default(),
        ));

        // No overstep accumulated => alpha ~ 0.
        app.update();

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert!(
            (tf.translation.x - 100.0).abs() < 1.0,
            "at alpha ~0, x should be ~100.0 (PreviousPosition) but got {}",
            tf.translation.x
        );
    }

    // ── Behavior 25: VisualOffset added ──

    #[test]
    fn visual_offset_added_to_translation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, derive_transform::<TestDrawLayer>);

        let entity = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(5.0, 5.0)),
                GlobalRotation2D::default(),
                GlobalScale2D::default(),
                VisualOffset(Vec3::new(0.0, 2.0, 0.0)),
                TestDrawLayer::A,
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(
            tf.translation,
            Vec3::new(5.0, 7.0, 0.0),
            "VisualOffset Y should be added to position Y"
        );
    }

    // ── Behavior 26: VisualOffset z stacks with DrawLayer z ──

    #[test]
    fn visual_offset_z_stacks_with_draw_layer_z() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, derive_transform::<TestDrawLayer>);

        let entity = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::ZERO),
                GlobalRotation2D::default(),
                GlobalScale2D::default(),
                VisualOffset(Vec3::new(0.0, 0.0, 0.5)),
                TestDrawLayer::B, // z = 1.0
                Transform::default(),
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert!(
            (tf.translation.z - 1.5).abs() < f32::EPSILON,
            "z should be DrawLayer(1.0) + VisualOffset(0.5) = 1.5, got {}",
            tf.translation.z
        );
    }

    // ── Behavior 27: Entity without DrawLayer not processed ──

    #[test]
    fn entity_without_draw_layer_not_processed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, derive_transform::<TestDrawLayer>);

        app.world_mut().spawn((
            GlobalPosition2D(Vec2::new(99.0, 99.0)),
            GlobalRotation2D::default(),
            GlobalScale2D::default(),
            Transform::from_xyz(1.0, 2.0, 3.0),
            // No TestDrawLayer
        ));

        tick(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert_eq!(
            tf.translation,
            Vec3::new(1.0, 2.0, 3.0),
            "Transform should be unchanged without a DrawLayer component"
        );
    }
}
