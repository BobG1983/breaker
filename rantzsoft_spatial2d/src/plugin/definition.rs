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
