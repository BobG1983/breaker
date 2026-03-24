//! `RantzSpatial2dPlugin` — registers all spatial systems.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    components::*,
    draw_layer::DrawLayer,
    propagation::*,
    systems::{
        apply_velocity::apply_velocity, compute_globals::compute_globals,
        derive_transform::derive_transform, propagate_position::propagate_position,
        propagate_rotation::propagate_rotation, propagate_scale::propagate_scale,
        save_previous::save_previous,
    },
};

/// Plugin that registers spatial propagation systems for a given `DrawLayer` type.
///
/// Generic over `D: DrawLayer` so each game can provide its own Z-ordering enum.
pub struct RantzSpatial2dPlugin<D: DrawLayer> {
    _marker: PhantomData<D>,
}

impl<D: DrawLayer> Default for RantzSpatial2dPlugin<D> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<D: DrawLayer> Plugin for RantzSpatial2dPlugin<D> {
    fn build(&self, app: &mut App) {
        app.register_type::<Position2D>()
            .register_type::<Rotation2D>()
            .register_type::<Scale2D>()
            .register_type::<PreviousPosition>()
            .register_type::<PreviousRotation>()
            .register_type::<PreviousScale>()
            .register_type::<InterpolateTransform2D>()
            .register_type::<VisualOffset>()
            .register_type::<PositionPropagation>()
            .register_type::<RotationPropagation>()
            .register_type::<ScalePropagation>()
            .register_type::<Velocity2D>()
            .register_type::<PreviousVelocity>()
            .register_type::<GlobalPosition2D>()
            .register_type::<GlobalRotation2D>()
            .register_type::<GlobalScale2D>()
            .register_type::<ApplyVelocity>()
            .add_systems(FixedFirst, save_previous)
            .add_systems(FixedUpdate, apply_velocity)
            .add_systems(
                RunFixedMainLoop,
                (
                    compute_globals,
                    derive_transform::<D>,
                    propagate_position::<D>,
                    propagate_rotation,
                    propagate_scale,
                )
                    .chain()
                    .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{
        ApplyVelocity, GlobalPosition2D, GlobalRotation2D, GlobalScale2D, PreviousVelocity,
        Velocity2D,
    };

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

    // ── Behavior 24: Plugin builds without panic ──

    #[test]
    fn plugin_builds_without_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.update();
        app.update();
    }

    // ── Behavior 35: Plugin registers compute_globals in FixedPostUpdate ──

    #[test]
    fn plugin_computes_globals_for_root_entity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

        let entity = app
            .world_mut()
            .spawn((Spatial2D, Position2D(Vec2::new(10.0, 20.0))))
            .id();

        tick(&mut app);

        let global_pos = app.world().get::<GlobalPosition2D>(entity).unwrap();
        assert_eq!(
            global_pos.0,
            Vec2::new(10.0, 20.0),
            "Plugin should run compute_globals: root GlobalPosition2D should equal Position2D"
        );
    }

    // ── Behavior 36: Plugin registers derive_transform in AfterFixedMainLoop ──

    #[test]
    fn plugin_derives_transform_from_globals() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

        let entity = app
            .world_mut()
            .spawn((
                Spatial2D,
                Position2D(Vec2::new(10.0, 20.0)),
                TestDrawLayer::A,
            ))
            .id();

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(
            tf.translation,
            Vec3::new(10.0, 20.0, 0.0),
            "Plugin should run derive_transform: Transform should match GlobalPosition2D"
        );
    }

    // ── Behavior 37: Plugin registers apply_velocity for entities with marker ──

    #[test]
    fn plugin_registers_apply_velocity_with_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

        let entity = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
                ApplyVelocity,
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        // dt = 1/64 = 0.015625, displacement = 400 * 0.015625 = 6.25
        assert!(
            (pos.0.y - 6.25).abs() < 1e-3,
            "Plugin should register apply_velocity: y should be ~6.25, got {}",
            pos.0.y
        );
    }

    #[test]
    fn plugin_apply_velocity_skips_without_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

        let entity = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        tick(&mut app);

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(
            pos.0,
            Vec2::ZERO,
            "Without ApplyVelocity marker, Position2D should be unchanged"
        );
    }

    // ── Regression: compute_globals must run before derive_transform ──

    #[test]
    fn velocity_driven_entity_transform_reflects_updated_position_after_one_tick() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

        // Entity with velocity and draw layer — uses full plugin pipeline without interpolation.
        let entity = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
                ApplyVelocity,
                GlobalPosition2D::default(),
                GlobalRotation2D::default(),
                GlobalScale2D::default(),
                Scale2D::default(),
                Rotation2D::default(),
                PreviousPosition::default(),
                PreviousRotation::default(),
                PreviousScale::default(),
                TestDrawLayer::A,
                Transform::default(),
            ))
            .id();

        // First tick: save_previous snapshots (0,0), apply_velocity moves
        // Position2D to (0, 6.25), compute_globals updates
        // GlobalPosition2D to (0, 6.25), derive_transform writes Transform
        // directly from GlobalPosition2D (no interpolation).
        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        // Without interpolation, derive_transform uses GlobalPosition2D directly.
        // dt = 1/64 = 0.015625, displacement = 400 * 0.015625 = 6.25.
        // If compute_globals runs AFTER derive_transform, GlobalPosition2D
        // will still be (0,0) when derive_transform reads it, so
        // Transform.translation.y will be 0.0 (incorrect).
        assert!(
            tf.translation.y > 1.0,
            "Transform.translation.y should reflect the velocity-updated position (~6.25), \
             but got {:.4} — compute_globals likely runs after derive_transform",
            tf.translation.y
        );
    }

    // ── Behavior 38: Plugin registers new type reflections ──

    #[test]
    fn plugin_registers_new_type_reflections() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.update();

        let registry = app.world().resource::<AppTypeRegistry>();
        let registry = registry.read();

        assert!(
            registry.get(std::any::TypeId::of::<Velocity2D>()).is_some(),
            "Velocity2D should be registered for reflection"
        );
        assert!(
            registry
                .get(std::any::TypeId::of::<PreviousVelocity>())
                .is_some(),
            "PreviousVelocity should be registered for reflection"
        );
        assert!(
            registry
                .get(std::any::TypeId::of::<GlobalPosition2D>())
                .is_some(),
            "GlobalPosition2D should be registered for reflection"
        );
        assert!(
            registry
                .get(std::any::TypeId::of::<GlobalRotation2D>())
                .is_some(),
            "GlobalRotation2D should be registered for reflection"
        );
        assert!(
            registry
                .get(std::any::TypeId::of::<GlobalScale2D>())
                .is_some(),
            "GlobalScale2D should be registered for reflection"
        );
    }
}
