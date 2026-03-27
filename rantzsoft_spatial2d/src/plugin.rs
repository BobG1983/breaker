//! `RantzSpatial2dPlugin` — registers all spatial systems.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    components::*,
    draw_layer::DrawLayer,
    propagation::*,
    systems::{
        apply_velocity::apply_velocity, compute_globals::compute_globals,
        derive_transform::derive_transform, save_previous::save_previous,
    },
};

/// System sets for ordering game systems relative to spatial pipeline stages.
///
/// Use these sets with `.before()` / `.after()` to guarantee your systems
/// observe consistent spatial state.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpatialSystems {
    /// `save_previous` snapshots current state into `Previous*` components.
    SavePrevious,
    /// `apply_velocity` advances `Position2D` by `Velocity2D * dt`.
    ApplyVelocity,
    /// `compute_globals` computes `Global*` components from the parent hierarchy.
    ComputeGlobals,
    /// `derive_transform` writes `Transform` from `Global*` components.
    DeriveTransform,
}

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
            .add_systems(
                FixedFirst,
                save_previous.in_set(SpatialSystems::SavePrevious),
            )
            .add_systems(
                FixedUpdate,
                apply_velocity.in_set(SpatialSystems::ApplyVelocity),
            )
            .add_systems(
                RunFixedMainLoop,
                (
                    compute_globals.in_set(SpatialSystems::ComputeGlobals),
                    derive_transform::<D>.in_set(SpatialSystems::DeriveTransform),
                )
                    .chain()
                    .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            );
    }
}

#[cfg(test)]
mod tests {
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

    // ── Behavior 24: Plugin builds without panic ──

    #[test]
    fn plugin_builds_without_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.update();
        app.update();
    }

    // ── Behavior 35: Plugin registers compute_globals in AfterFixedMainLoop ──

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
        let guard = registry.read();

        assert!(
            guard.get(std::any::TypeId::of::<Velocity2D>()).is_some(),
            "Velocity2D should be registered for reflection"
        );
        assert!(
            guard
                .get(std::any::TypeId::of::<PreviousVelocity>())
                .is_some(),
            "PreviousVelocity should be registered for reflection"
        );
        assert!(
            guard
                .get(std::any::TypeId::of::<GlobalPosition2D>())
                .is_some(),
            "GlobalPosition2D should be registered for reflection"
        );
        assert!(
            guard
                .get(std::any::TypeId::of::<GlobalRotation2D>())
                .is_some(),
            "GlobalRotation2D should be registered for reflection"
        );
        assert!(
            guard
                .get(std::any::TypeId::of::<GlobalScale2D>())
                .is_some(),
            "GlobalScale2D should be registered for reflection"
        );
        drop(guard);
    }

    // ── Behavior: SpatialSystems enum is public and has all four variants ──

    #[test]
    fn spatial_systems_enum_has_all_four_variants() {
        // Verify all four variants can be instantiated and the enum derives
        // SystemSet, Debug, Clone, PartialEq, Eq, Hash.
        let save = SpatialSystems::SavePrevious;
        let apply = SpatialSystems::ApplyVelocity;
        let compute = SpatialSystems::ComputeGlobals;
        let derive = SpatialSystems::DeriveTransform;

        // Debug
        let _debug = format!("{save:?} {apply:?} {compute:?} {derive:?}");

        // Clone
        let cloned = save.clone();

        // PartialEq + Eq
        assert_eq!(save, cloned, "SpatialSystems should derive PartialEq + Eq");
        assert_ne!(save, apply, "Different variants should not be equal");

        // Hash — verify it compiles by inserting into a HashSet
        let mut set = std::collections::HashSet::new();
        set.insert(save);
        set.insert(apply);
        set.insert(compute);
        set.insert(derive);
        assert_eq!(set.len(), 4, "All four variants should hash distinctly");
    }

    // ── Behavior: save_previous runs in SpatialSystems::SavePrevious set ──

    /// Resource used to capture `PreviousPosition` observed by the
    /// after-`SavePrevious` test system.
    #[derive(Resource, Default)]
    struct CapturedPreviousPosition(Vec2);

    /// Overwrites `PreviousPosition` with a sentinel BEFORE `SavePrevious`.
    /// If `save_previous` is properly in the `SavePrevious` set, this runs
    /// first and `save_previous` overwrites the sentinel with the real value.
    fn set_previous_position_sentinel(
        mut query: Query<&mut PreviousPosition, With<InterpolateTransform2D>>,
    ) {
        for mut prev in &mut query {
            prev.0 = Vec2::new(999.0, 999.0);
        }
    }

    /// Reads `PreviousPosition` AFTER `SavePrevious`. If ordering is correct,
    /// `save_previous` has already overwritten the sentinel.
    fn capture_previous_position_after_save(
        query: Query<&PreviousPosition, With<InterpolateTransform2D>>,
        mut captured: ResMut<CapturedPreviousPosition>,
    ) {
        for prev in &query {
            captured.0 = prev.0;
        }
    }

    #[test]
    fn save_previous_runs_in_save_previous_set() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.init_resource::<CapturedPreviousPosition>();

        // Add sentinel-writer BEFORE the SavePrevious set and reader AFTER.
        app.add_systems(
            FixedFirst,
            set_previous_position_sentinel.before(SpatialSystems::SavePrevious),
        );
        app.add_systems(
            FixedFirst,
            capture_previous_position_after_save.after(SpatialSystems::SavePrevious),
        );

        // Spawn entity with InterpolateTransform2D so save_previous processes it.
        // Position2D = (10.0, 20.0), PreviousPosition starts at (0.0, 0.0).
        // No Spatial2D or GlobalPosition2D — save_previous falls back to local
        // Position2D when GlobalPosition2D is absent.
        app.world_mut().spawn((
            InterpolateTransform2D,
            Position2D(Vec2::new(10.0, 20.0)),
            PreviousPosition(Vec2::ZERO),
            Rotation2D::default(),
            PreviousRotation::default(),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedPreviousPosition>();
        // If save_previous is in SavePrevious set:
        //   sentinel (999, 999) -> save_previous overwrites with (10, 20) -> captured = (10, 20)
        // If save_previous is NOT in set (insertion-order: save_previous, sentinel, capture):
        //   save_previous copies (10, 20) -> sentinel overwrites to (999, 999) -> captured = (999, 999)
        assert_eq!(
            captured.0,
            Vec2::new(10.0, 20.0),
            "After SpatialSystems::SavePrevious, PreviousPosition should be (10, 20) \
             (snapshotted by save_previous), but got {:?} — save_previous is likely not \
             in the SavePrevious system set",
            captured.0,
        );
    }

    #[test]
    fn after_save_previous_set_compiles_as_ordering_constraint() {
        // Edge case: verifying .after(SpatialSystems::SavePrevious) does not
        // panic at plugin build time.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.add_systems(FixedFirst, (|| {}).after(SpatialSystems::SavePrevious));
        // If this update does not panic, the set is usable for ordering.
        app.update();
    }

    // ── Behavior: apply_velocity runs in SpatialSystems::ApplyVelocity set ──

    /// Resource to capture position observed by the before-`ApplyVelocity` system.
    #[derive(Resource, Default)]
    struct CapturedPositionBefore(Vec2);

    /// Resource to capture position observed by the after-`ApplyVelocity` system.
    #[derive(Resource, Default)]
    struct CapturedPositionAfter(Vec2);

    /// Reads `Position2D` BEFORE `ApplyVelocity` and writes a sentinel to force
    /// a known starting state for `apply_velocity`.
    fn capture_position_before_apply_velocity(
        mut query: Query<&mut Position2D, With<ApplyVelocity>>,
        mut captured: ResMut<CapturedPositionBefore>,
    ) {
        for mut pos in &mut query {
            captured.0 = pos.0;
            // Overwrite to a known sentinel so apply_velocity starts from here.
            pos.0 = Vec2::new(100.0, 100.0);
        }
    }

    /// Reads `Position2D` AFTER `ApplyVelocity`.
    fn capture_position_after_apply_velocity(
        query: Query<&Position2D, With<ApplyVelocity>>,
        mut captured: ResMut<CapturedPositionAfter>,
    ) {
        for pos in &query {
            captured.0 = pos.0;
        }
    }

    #[test]
    fn apply_velocity_runs_in_apply_velocity_set() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.init_resource::<CapturedPositionBefore>();
        app.init_resource::<CapturedPositionAfter>();

        app.add_systems(
            FixedUpdate,
            capture_position_before_apply_velocity.before(SpatialSystems::ApplyVelocity),
        );
        app.add_systems(
            FixedUpdate,
            capture_position_after_apply_velocity.after(SpatialSystems::ApplyVelocity),
        );

        // Entity at origin with upward velocity.
        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            ApplyVelocity,
        ));

        tick(&mut app);

        let after = app.world().resource::<CapturedPositionAfter>();
        // If apply_velocity is in ApplyVelocity set:
        //   before-system: pos = (0,0) captured, then set to (100, 100)
        //   apply_velocity: pos = (100, 100) + (0, 400)*(1/64) = (100, 106.25)
        //   after-system: captured = (100, 106.25)
        //
        // If apply_velocity is NOT in set (insertion order: apply_velocity first):
        //   apply_velocity: pos = (0,0) + (0,6.25) = (0, 6.25)
        //   before-system: captured (0, 6.25), set to (100, 100)
        //   after-system: captured = (100, 100) — NOT ~106.25
        assert!(
            (after.0.y - 106.25).abs() < 1e-3,
            "After SpatialSystems::ApplyVelocity, Position2D.y should be ~106.25 \
             (sentinel 100 + 400*(1/64)), but got {:.4} — apply_velocity is likely not \
             in the ApplyVelocity system set",
            after.0.y,
        );
    }

    /// Reads `Position2D` without mutating it, for use as a before-observer.
    fn read_position_before_apply_velocity(
        query: Query<&Position2D, With<ApplyVelocity>>,
        mut captured: ResMut<CapturedPositionBefore>,
    ) {
        for pos in &query {
            captured.0 = pos.0;
        }
    }

    #[test]
    fn before_apply_velocity_set_sees_pre_velocity_position() {
        // Edge case: a system configured .before(SpatialSystems::ApplyVelocity)
        // should see Position2D before apply_velocity mutates it.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());
        app.init_resource::<CapturedPositionBefore>();

        app.add_systems(
            FixedUpdate,
            read_position_before_apply_velocity.before(SpatialSystems::ApplyVelocity),
        );

        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            Velocity2D(Vec2::new(0.0, 400.0)),
            ApplyVelocity,
        ));

        tick(&mut app);

        let before = app.world().resource::<CapturedPositionBefore>();
        // If apply_velocity is in ApplyVelocity set:
        //   before-system runs first: sees Position2D = (0, 0)
        //
        // If apply_velocity is NOT in set (insertion order: apply_velocity first):
        //   apply_velocity: pos = (0, 6.25)
        //   before-system: sees pos = (0, 6.25) — NOT (0, 0)
        assert_eq!(
            before.0,
            Vec2::ZERO,
            "A system .before(SpatialSystems::ApplyVelocity) should see Position2D \
             before velocity is applied (Vec2::ZERO), but got {:?} — apply_velocity \
             is likely not in the ApplyVelocity system set",
            before.0,
        );
    }

    // ── Behavior: compute_globals and derive_transform sets usable for ordering ──

    #[test]
    fn compute_globals_and_derive_transform_sets_usable_for_ordering() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

        // Add systems ordered relative to ComputeGlobals and DeriveTransform
        // in RunFixedMainLoop. If the sets are registered, the app builds
        // and ticks without panics.
        app.add_systems(
            RunFixedMainLoop,
            (|| {}).after(SpatialSystems::ComputeGlobals),
        );
        app.add_systems(
            RunFixedMainLoop,
            (|| {}).after(SpatialSystems::DeriveTransform),
        );

        // Tick twice to exercise the schedule.
        tick(&mut app);
        tick(&mut app);
    }

    #[test]
    fn existing_regression_test_still_passes_with_system_sets() {
        // This mirrors the existing regression test to confirm that introducing
        // system sets does not break the compute_globals-before-derive_transform
        // ordering.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzSpatial2dPlugin::<TestDrawLayer>::default());

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

        tick(&mut app);

        let tf = app.world().get::<Transform>(entity).unwrap();
        assert!(
            tf.translation.y > 1.0,
            "Transform.translation.y should reflect velocity-updated position (~6.25), \
             but got {:.4} — compute_globals → derive_transform ordering may be broken",
            tf.translation.y,
        );
    }

    // ── Behavior: prelude re-exports all public API types ──

    #[test]
    fn prelude_re_exports_spatial_systems() {
        // Verify SpatialSystems is importable from the prelude.
        use crate::prelude::SpatialSystems as PreludeSpatialSystems;

        let set = PreludeSpatialSystems::SavePrevious;
        // Prove it's the same type by comparing with the direct import.
        assert_eq!(
            set,
            SpatialSystems::SavePrevious,
            "SpatialSystems from prelude should be the same type as the direct import"
        );
    }
}
