//! Shared helpers for effect fire/reverse functions.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt},
        resources::BoltConfig,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer, WALL_LAYER,
        rng::GameRng,
    },
};

/// Returns the entity's [`Position2D`] value, or [`Vec2::ZERO`] if absent.
pub(crate) fn entity_position(world: &World, entity: Entity) -> Vec2 {
    world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0)
}

/// Computes the effective range for an area-of-effect based on stacks.
///
/// Formula: `base_range + u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX) as f32 * range_per_level`
///
/// For `stacks >= 1`, the extra range is `(stacks - 1) * range_per_level` (capped at `u16::MAX` levels).
/// For `stacks == 0`, `saturating_sub(1)` wraps to `u32::MAX` which saturates to `u16::MAX`.
pub(crate) fn effective_range(base_range: f32, range_per_level: f32, stacks: u32) -> f32 {
    let extra = u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX);
    base_range + f32::from(extra) * range_per_level
}

/// Spawn an extra bolt entity with full physics components at the given position.
///
/// Reads [`BoltConfig`] for radius/speed values and [`GameRng`] for a random
/// velocity direction. Returns the spawned entity ID. Callers insert
/// effect-specific markers (e.g., `ChainBoltMarker`, `PhantomBoltMarker`) after.
pub(crate) fn spawn_extra_bolt(world: &mut World, spawn_pos: Vec2) -> Entity {
    let config = world.resource::<BoltConfig>();
    let radius = config.radius;
    let base_speed = config.base_speed;
    let min_speed = config.min_speed;
    let max_speed = config.max_speed;

    let angle = {
        let mut rng = world.resource_mut::<GameRng>();
        rng.0.random_range(0.0..std::f32::consts::TAU)
    };
    let direction = Vec2::new(angle.cos(), angle.sin());
    let velocity = direction * base_speed;

    world
        .spawn((
            (
                Bolt,
                ExtraBolt,
                Position2D(spawn_pos),
                PreviousPosition(spawn_pos),
                Scale2D {
                    x: radius,
                    y: radius,
                },
                PreviousScale {
                    x: radius,
                    y: radius,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius)),
            ),
            (
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
                Velocity2D(velocity),
                BoltBaseSpeed(base_speed),
                BoltMinSpeed(min_speed),
                BoltMaxSpeed(max_speed),
                BoltRadius(radius),
                CleanupOnNodeExit,
                GameDrawLayer::Bolt,
            ),
        ))
        .id()
}
