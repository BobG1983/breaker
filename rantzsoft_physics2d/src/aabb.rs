//! Axis-aligned bounding box for 2D collision detection.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Spatial2D;

/// Axis-aligned bounding box defined by center and half-extents.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[require(Spatial2D)]
pub struct Aabb2D {
    /// Center position of the bounding box.
    pub center: Vec2,
    /// Half-extents (half-width, half-height) from center to each edge.
    pub half_extents: Vec2,
}

impl Aabb2D {
    /// Creates a new `Aabb2D` from a center point and half-extents.
    #[must_use]
    pub const fn new(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            center,
            half_extents,
        }
    }

    /// Creates an `Aabb2D` from minimum and maximum corner points.
    #[must_use]
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            center: (min + max) / 2.0,
            half_extents: (max - min) / 2.0,
        }
    }

    /// Returns the minimum corner (center - half-extents).
    #[must_use]
    pub fn min(&self) -> Vec2 {
        self.center - self.half_extents
    }

    /// Returns the maximum corner (center + half-extents).
    #[must_use]
    pub fn max(&self) -> Vec2 {
        self.center + self.half_extents
    }

    /// Returns `true` if the point is inside or on the boundary of this AABB.
    #[must_use]
    pub fn contains_point(&self, point: Vec2) -> bool {
        let min = self.min();
        let max = self.max();
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    /// Returns `true` if this AABB overlaps with another (inclusive of edges).
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        !(self.max().x < other.min().x
            || self.min().x > other.max().x
            || self.max().y < other.min().y
            || self.min().y > other.max().y)
    }

    /// Returns a new `Aabb2D` with half-extents grown by `amount` on each axis.
    #[must_use]
    pub fn expand_by(&self, amount: f32) -> Self {
        Self {
            center: self.center,
            half_extents: self.half_extents + Vec2::splat(amount),
        }
    }

    /// Returns `true` if `other` is fully contained within this AABB.
    #[must_use]
    pub fn contains_aabb(&self, other: &Self) -> bool {
        let self_min = self.min();
        let self_max = self.max();
        let other_min = other.min();
        let other_max = other.max();
        other_min.x >= self_min.x
            && other_min.y >= self_min.y
            && other_max.x <= self_max.x
            && other_max.y <= self_max.y
    }

    /// Returns the smallest `Aabb2D` that contains both `self` and `other`.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self::from_min_max(self.min().min(other.min()), self.max().max(other.max()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_center_and_half_extents() {
        let aabb = Aabb2D::new(Vec2::new(5.0, 10.0), Vec2::new(2.0, 3.0));
        assert_eq!(aabb.center, Vec2::new(5.0, 10.0));
        assert_eq!(aabb.half_extents, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn from_min_max_computes_center_and_half_extents() {
        let aabb = Aabb2D::from_min_max(Vec2::new(0.0, 0.0), Vec2::new(10.0, 6.0));
        assert_eq!(aabb.center, Vec2::new(5.0, 3.0));
        assert_eq!(aabb.half_extents, Vec2::new(5.0, 3.0));
    }

    #[test]
    fn min_returns_center_minus_half_extents() {
        let aabb = Aabb2D::new(Vec2::new(5.0, 5.0), Vec2::new(2.0, 3.0));
        assert_eq!(aabb.min(), Vec2::new(3.0, 2.0));
    }

    #[test]
    fn max_returns_center_plus_half_extents() {
        let aabb = Aabb2D::new(Vec2::new(5.0, 5.0), Vec2::new(2.0, 3.0));
        assert_eq!(aabb.max(), Vec2::new(7.0, 8.0));
    }

    #[test]
    fn contains_point_true_for_interior_point() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(aabb.contains_point(Vec2::new(5.0, 5.0)));
    }

    #[test]
    fn contains_point_false_for_exterior_point() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        assert!(!aabb.contains_point(Vec2::new(15.0, 0.0)));
    }

    #[test]
    fn contains_point_true_at_boundary() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        // Point exactly on the +X edge
        assert!(aabb.contains_point(Vec2::new(10.0, 0.0)));
    }

    #[test]
    fn overlaps_true_when_overlapping() {
        let a = Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0));
        let b = Aabb2D::new(Vec2::new(3.0, 3.0), Vec2::new(5.0, 5.0));
        assert!(a.overlaps(&b));
    }

    #[test]
    fn overlaps_false_when_separated() {
        let a = Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0));
        let b = Aabb2D::new(Vec2::new(20.0, 20.0), Vec2::new(5.0, 5.0));
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn overlaps_true_when_touching_edges() {
        // a spans [-5, 5] on X; b spans [5, 15] on X. They share the X=5 edge.
        let a = Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0));
        let b = Aabb2D::new(Vec2::new(10.0, 0.0), Vec2::new(5.0, 5.0));
        assert!(a.overlaps(&b));
    }

    #[test]
    fn expand_by_grows_half_extents_and_preserves_center() {
        let aabb = Aabb2D::new(Vec2::new(1.0, 2.0), Vec2::new(5.0, 5.0));
        let expanded = aabb.expand_by(2.0);
        assert_eq!(expanded.center, Vec2::new(1.0, 2.0));
        assert_eq!(expanded.half_extents, Vec2::new(7.0, 7.0));
    }

    #[test]
    fn contains_aabb_true_when_inner_fully_inside() {
        let outer = Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        let inner = Aabb2D::new(Vec2::ZERO, Vec2::new(3.0, 3.0));
        assert!(outer.contains_aabb(&inner));
    }

    #[test]
    fn contains_aabb_false_when_partially_outside() {
        let outer = Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0));
        // Inner shifted right so its max.x = 9 + 3 = 12, outside outer's max.x = 10
        let inner = Aabb2D::new(Vec2::new(9.0, 0.0), Vec2::new(3.0, 3.0));
        assert!(!outer.contains_aabb(&inner));
    }

    #[test]
    fn merge_produces_enclosing_aabb() {
        let a = Aabb2D::new(Vec2::new(-5.0, 0.0), Vec2::new(3.0, 3.0));
        let b = Aabb2D::new(Vec2::new(5.0, 0.0), Vec2::new(3.0, 3.0));

        let merged = a.merge(&b);

        // a spans [-8, -2] on X, b spans [2, 8] on X
        // Merged should span [-8, 8] → center=0, half_x=8
        // Both span [-3, 3] on Y → center=0, half_y=3
        assert_eq!(merged.center, Vec2::new(0.0, 0.0));
        assert_eq!(merged.half_extents, Vec2::new(8.0, 3.0));
    }

    // ── Behavior 6: Aabb2D as Component with #[require(Spatial2D)] ──

    #[test]
    fn aabb2d_spawned_alone_gets_spatial2d_required_components() {
        use rantzsoft_spatial2d::components::{Position2D, Rotation2D, Scale2D};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn(Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)))
            .id();
        app.update();

        let world = app.world();
        assert!(
            world.get::<Position2D>(entity).is_some(),
            "missing Position2D"
        );
        assert!(
            world.get::<Rotation2D>(entity).is_some(),
            "missing Rotation2D"
        );
        assert!(world.get::<Scale2D>(entity).is_some(), "missing Scale2D");
    }

    #[test]
    fn aabb2d_preserves_explicit_position_when_spawned_alongside() {
        use rantzsoft_spatial2d::components::Position2D;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app
            .world_mut()
            .spawn((
                Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
                Position2D(Vec2::new(5.0, 5.0)),
            ))
            .id();
        app.update();

        let pos = app.world().get::<Position2D>(entity).unwrap();
        assert_eq!(
            pos.0,
            Vec2::new(5.0, 5.0),
            "explicit Position2D should not be overwritten by default"
        );
    }
}
