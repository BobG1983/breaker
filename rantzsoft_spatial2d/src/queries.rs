//! Spatial query data types for Bevy ECS.

use bevy::ecs::query::QueryData;

use crate::components::{
    BaseSpeed, GlobalPosition2D, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed,
    Position2D, Velocity2D,
};

/// Core spatial query data: position, velocity, global position, and velocity
/// constraint parameters.
///
/// Does not include `Entity` — add that at the game-level query. Use
/// `Query<SpatialData, With<Spatial>>` to filter to entities with
/// velocity constraint data.
///
/// Optional fields degrade gracefully: `None` means no constraint for that
/// bound/angle.
#[derive(QueryData)]
#[query_data(mutable)]
pub struct SpatialData {
    /// Mutable world-space position.
    pub position: &'static mut Position2D,
    /// Mutable velocity vector.
    pub velocity: &'static mut Velocity2D,
    /// Read-only global position (from parent hierarchy).
    pub global_position: &'static GlobalPosition2D,
    /// Base speed before multipliers.
    pub base_speed: &'static BaseSpeed,
    /// Optional minimum speed constraint.
    pub min_speed: Option<&'static MinSpeed>,
    /// Optional maximum speed constraint.
    pub max_speed: Option<&'static MaxSpeed>,
    /// Optional minimum angle from horizontal.
    pub min_angle_h: Option<&'static MinAngleHorizontal>,
    /// Optional minimum angle from vertical.
    pub min_angle_v: Option<&'static MinAngleVertical>,
}
