//! Propagates `Position2D` + `DrawLayer` to `Transform.translation` with interpolation and visual
//! offset.

use bevy::prelude::*;

use crate::{
    components::{InterpolateTransform2D, Position2D, PreviousPosition, VisualOffset},
    draw_layer::DrawLayer,
    propagation::PositionPropagation,
};

type PropagatePositionQuery<'w, 's, D> = Query<
    'w,
    's,
    (
        &'static Position2D,
        &'static D,
        &'static mut Transform,
        Option<&'static InterpolateTransform2D>,
        Option<&'static PreviousPosition>,
        Option<&'static VisualOffset>,
        Option<&'static PositionPropagation>,
        Option<&'static ChildOf>,
    ),
>;

/// Writes `Transform.translation` from `Position2D`, draw-layer Z, and optional
/// `VisualOffset`. When `InterpolateTransform2D` is present, lerps between
/// `PreviousPosition` and `Position2D` using the fixed-timestep overstep fraction.
///
/// For children with `PositionPropagation::Absolute`, counteracts the parent's
/// position so Bevy's `TransformPropagate` produces the correct world position.
pub fn propagate_position<D: DrawLayer>(
    time: Res<Time<Fixed>>,
    mut query: PropagatePositionQuery<D>,
    parent_positions: Query<&Position2D>,
) {
    let alpha = time.overstep_fraction();

    for (pos, layer, mut transform, interp, prev_pos, offset, prop, child_of) in &mut query {
        // Interpolate or use current position.
        let base = if interp.is_some() {
            if let Some(prev) = prev_pos {
                prev.0.lerp(pos.0, alpha)
            } else {
                pos.0
            }
        } else {
            pos.0
        };

        let mut result = base.extend(layer.z());

        // Parent/child: counteract parent's position for Absolute propagation.
        if let Some(child_of) = child_of
            && prop.is_some_and(|p| *p == PositionPropagation::Absolute)
            && let Ok(parent_pos) = parent_positions.get(child_of.parent())
        {
            result.x -= parent_pos.0.x;
            result.y -= parent_pos.0.y;
        }

        // Apply visual offset.
        if let Some(offset) = offset {
            result += offset.0;
        }

        transform.translation = result;
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

    fn tick_fixed(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 6: Basic position to transform (no interpolation) ──

    #[test]
    fn basic_position_to_transform_no_interpolation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_position::<TestDrawLayer>);

        app.world_mut().spawn((
            Position2D(Vec2::new(10.0, 20.0)),
            TestDrawLayer::B,
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
            tf.translation,
            Vec3::new(10.0, 20.0, 1.0),
            "Transform should be (10, 20, z=1.0 from TestDrawLayer::B)"
        );
    }

    // ── Behavior 7: Interpolated position at alpha=0.5 ──

    #[test]
    fn interpolated_position_at_half_alpha() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, propagate_position::<TestDrawLayer>);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousPosition(Vec2::new(0.0, 0.0)),
            Position2D(Vec2::new(10.0, 0.0)),
            TestDrawLayer::A,
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

        assert!(
            (tf.translation.x - 5.0).abs() < 0.5,
            "at alpha ~0.5, x should be ~5.0 but got {}",
            tf.translation.x
        );
    }

    // ── Behavior 8: Alpha=0 uses PreviousPosition ──

    #[test]
    fn alpha_zero_uses_previous_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, propagate_position::<TestDrawLayer>);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousPosition(Vec2::new(100.0, 0.0)),
            Position2D(Vec2::new(200.0, 0.0)),
            TestDrawLayer::A,
            Transform::default(),
        ));

        // No overstep accumulated => alpha ≈ 0.
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

    // ── Behavior 9: High alpha uses mostly current Position2D ──

    #[test]
    fn high_alpha_uses_mostly_current_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PostUpdate, propagate_position::<TestDrawLayer>);

        app.world_mut().spawn((
            InterpolateTransform2D,
            PreviousPosition(Vec2::new(100.0, 0.0)),
            Position2D(Vec2::new(200.0, 0.0)),
            TestDrawLayer::A,
            Transform::default(),
        ));

        // Accumulate 1.9 timesteps. The fixed loop runs 1 tick (consuming 1.0),
        // leaving 0.9 timestep as overstep. overstep_fraction() = 0.9.
        // lerp(100.0, 200.0, 0.9) = 190.0
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

        // At alpha ~0.9: lerp(100, 200, 0.9) = 190.0
        assert!(
            (tf.translation.x - 190.0).abs() < 5.0,
            "at alpha ~0.9, x should be ~190.0 but got {}",
            tf.translation.x
        );
    }

    // ── Behavior 10: VisualOffset added ──

    #[test]
    fn visual_offset_added_to_translation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_position::<TestDrawLayer>);

        app.world_mut().spawn((
            Position2D(Vec2::new(5.0, 5.0)),
            VisualOffset(Vec3::new(0.0, 2.0, 0.0)),
            TestDrawLayer::A,
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
            tf.translation,
            Vec3::new(5.0, 7.0, 0.0),
            "VisualOffset Y should be added to position Y"
        );
    }

    // ── Behavior 11: VisualOffset z stacks with DrawLayer z ──

    #[test]
    fn visual_offset_z_stacks_with_draw_layer_z() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_position::<TestDrawLayer>);

        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            VisualOffset(Vec3::new(0.0, 0.0, 0.5)),
            TestDrawLayer::B, // z = 1.0
            Transform::default(),
        ));

        tick_fixed(&mut app);

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert!(
            (tf.translation.z - 1.5).abs() < f32::EPSILON,
            "z should be DrawLayer(1.0) + VisualOffset(0.5) = 1.5, got {}",
            tf.translation.z
        );
    }

    // ── Behavior 12: Entity without DrawLayer not processed ──

    #[test]
    fn entity_without_draw_layer_not_processed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, propagate_position::<TestDrawLayer>);

        app.world_mut().spawn((
            Position2D(Vec2::new(99.0, 99.0)),
            Transform::from_xyz(1.0, 2.0, 3.0),
            // No TestDrawLayer
        ));

        tick_fixed(&mut app);

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

    // ── Behavior 13: Parent/child Relative position ──

    #[test]
    fn parent_child_relative_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_systems(
            FixedUpdate,
            propagate_position::<TestDrawLayer>
                .before(bevy::transform::TransformSystems::Propagate),
        );

        let parent = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(10.0, 0.0)),
                TestDrawLayer::A,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Position2D(Vec2::new(5.0, 0.0)),
                PositionPropagation::Relative,
                TestDrawLayer::A,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // Run multiple ticks to ensure both propagation passes complete.
        tick_fixed(&mut app);
        tick_fixed(&mut app);

        let global = app
            .world()
            .get::<GlobalTransform>(child)
            .expect("child should have GlobalTransform");

        assert!(
            (global.translation().x - 15.0).abs() < 0.1,
            "child global x should be parent(10) + child(5) = 15, got {}",
            global.translation().x
        );
    }

    // ── Behavior 14: Parent/child Absolute position ──

    #[test]
    fn parent_child_absolute_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_systems(
            FixedUpdate,
            propagate_position::<TestDrawLayer>
                .before(bevy::transform::TransformSystems::Propagate),
        );

        let parent = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(10.0, 0.0)),
                TestDrawLayer::A,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let child = app
            .world_mut()
            .spawn((
                ChildOf(parent),
                Position2D(Vec2::new(5.0, 0.0)),
                PositionPropagation::Absolute,
                TestDrawLayer::A,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // Run multiple ticks to ensure both propagation passes complete.
        tick_fixed(&mut app);
        tick_fixed(&mut app);

        let global = app
            .world()
            .get::<GlobalTransform>(child)
            .expect("child should have GlobalTransform");

        assert!(
            (global.translation().x - 5.0).abs() < 0.1,
            "child global x should be absolute 5.0 (ignoring parent), got {}",
            global.translation().x
        );
    }
}
