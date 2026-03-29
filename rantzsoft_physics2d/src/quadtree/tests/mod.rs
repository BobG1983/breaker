mod basic_ops_tests;
mod cast_tests;
mod layer_filter_tests;

use bevy::{ecs::world::World, prelude::*};

use crate::{aabb::Aabb2D, quadtree::Quadtree};

/// Helper: creates a quadtree covering [-500, 500] on both axes with default
/// leaf capacity 8 and max depth 8.
pub(super) fn test_tree() -> Quadtree {
    Quadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(500.0, 500.0)), 8, 8)
}

/// Helper: creates a small `Aabb2D` centered at the given position.
pub(super) fn small_aabb(x: f32, y: f32) -> Aabb2D {
    Aabb2D::new(Vec2::new(x, y), Vec2::new(1.0, 1.0))
}

/// Helper: spawns N distinct entities from a `World` for use as quadtree keys.
pub(super) fn spawn_entities(count: usize) -> Vec<Entity> {
    let mut world = World::new();
    (0..count).map(|_| world.spawn_empty().id()).collect()
}
