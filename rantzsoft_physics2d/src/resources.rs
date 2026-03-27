//! Physics resources for the `RantzPhysics2dPlugin`.

use bevy::prelude::*;

use crate::{aabb::Aabb2D, quadtree::Quadtree};

/// Shared quadtree spatial index resource, maintained by `maintain_quadtree`.
#[derive(Resource)]
pub struct CollisionQuadtree {
    /// The underlying quadtree spatial index.
    pub quadtree: Quadtree,
}

impl CollisionQuadtree {
    /// Creates a new `CollisionQuadtree` with the given world bounds.
    #[must_use]
    pub const fn new(bounds: Aabb2D, max_items_per_leaf: usize, max_depth: usize) -> Self {
        Self {
            quadtree: Quadtree::new(bounds, max_items_per_leaf, max_depth),
        }
    }
}

impl Default for CollisionQuadtree {
    fn default() -> Self {
        Self::new(Aabb2D::new(Vec2::ZERO, Vec2::new(600.0, 400.0)), 8, 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 13: CollisionQuadtree wraps Quadtree and is a Resource ──

    #[test]
    fn collision_quadtree_new_creates_empty_quadtree() {
        let cq = CollisionQuadtree::new(Aabb2D::new(Vec2::ZERO, Vec2::new(600.0, 400.0)), 8, 8);
        assert_eq!(cq.quadtree.len(), 0);
        assert!(cq.quadtree.is_empty());
    }

    #[test]
    fn collision_quadtree_default_has_sensible_bounds() {
        let cq = CollisionQuadtree::default();
        assert_eq!(cq.quadtree.len(), 0);
        assert!(cq.quadtree.is_empty());
    }
}
