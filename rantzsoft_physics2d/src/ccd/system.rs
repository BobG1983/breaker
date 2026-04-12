//! Continuous collision detection: ray-vs-AABB sweep tests.

use bevy::prelude::*;

use crate::aabb::Aabb2D;

/// Maximum number of bounces resolved per moving object per frame.
///
/// Prevents infinite loops in degenerate geometries.
pub const MAX_BOUNCES: u32 = 4;

/// Sub-pixel separation gap applied after each collision.
///
/// The moving object is placed this far outside the target's expanded AABB
/// to prevent floating-point touching on the next sweep.
pub const CCD_EPSILON: f32 = 0.01;

/// Result of a ray-vs-expanded-AABB intersection test.
pub struct RayHit {
    /// Distance along the ray to the entry point.
    pub distance: f32,
    /// Outward face normal at the entry point.
    pub normal:   Vec2,
}

impl RayHit {
    /// Computes the safe position along the ray, offset from the collision
    /// surface by the internal CCD epsilon to prevent floating-point touching.
    #[must_use]
    pub fn safe_position(&self, origin: Vec2, direction: Vec2) -> Vec2 {
        origin + direction * (self.distance - CCD_EPSILON).max(0.0)
    }

    /// Returns the remaining travel distance after reaching the hit point.
    #[must_use]
    pub fn remaining(&self, max_dist: f32) -> f32 {
        (max_dist - self.distance).max(0.0)
    }
}

/// Result of a swept circle (or ray) cast against the quadtree.
pub struct SweepHit {
    /// The entity that was hit.
    pub entity:    Entity,
    /// Safe position for the moving object just before contact.
    pub position:  Vec2,
    /// Outward face normal at the contact point.
    pub normal:    Vec2,
    /// Remaining travel distance after the hit.
    pub remaining: f32,
}

/// Casts a ray against an `Aabb2D` and returns the entry distance and face normal.
///
/// The AABB should already be Minkowski-expanded by the object radius so that
/// a point-ray test is equivalent to a circle-vs-rectangle test.
///
/// Returns `None` if the ray misses, the origin is inside the AABB, or the
/// hit is beyond `max_dist`.
#[must_use]
pub fn ray_vs_aabb(origin: Vec2, direction: Vec2, max_dist: f32, aabb: &Aabb2D) -> Option<RayHit> {
    let aabb_min = aabb.min();
    let aabb_max = aabb.max();

    let mut tmin = 0.0_f32;
    let mut tmax = max_dist;
    let mut normal = Vec2::ZERO;

    // X slab
    if direction.x.abs() < f32::EPSILON {
        if origin.x < aabb_min.x || origin.x > aabb_max.x {
            return None;
        }
    } else {
        let inv_d = direction.x.recip();
        let t1 = (aabb_min.x - origin.x) * inv_d;
        let t2 = (aabb_max.x - origin.x) * inv_d;
        let (t_near, t_far, near_normal) = if t1 < t2 {
            (t1, t2, Vec2::NEG_X)
        } else {
            (t2, t1, Vec2::X)
        };
        if t_near > tmin {
            tmin = t_near;
            normal = near_normal;
        }
        tmax = tmax.min(t_far);
        if tmin > tmax {
            return None;
        }
    }

    // Y slab
    if direction.y.abs() < f32::EPSILON {
        if origin.y < aabb_min.y || origin.y > aabb_max.y {
            return None;
        }
    } else {
        let inv_d = direction.y.recip();
        let t1 = (aabb_min.y - origin.y) * inv_d;
        let t2 = (aabb_max.y - origin.y) * inv_d;
        let (t_near, t_far, near_normal) = if t1 < t2 {
            (t1, t2, Vec2::NEG_Y)
        } else {
            (t2, t1, Vec2::Y)
        };
        if t_near > tmin {
            tmin = t_near;
            normal = near_normal;
        }
        tmax = tmax.min(t_far);
        if tmin > tmax {
            return None;
        }
    }

    // Origin inside AABB (tmin == 0 means the ray starts overlapping)
    if tmin <= 0.0 {
        return None;
    }

    Some(RayHit {
        distance: tmin,
        normal,
    })
}

/// Reflects a velocity vector off a surface normal.
///
/// Standard reflection formula: `v - 2(v*n)n`. Preserves speed (magnitude).
#[must_use]
pub fn reflect(velocity: Vec2, normal: Vec2) -> Vec2 {
    velocity - 2.0 * velocity.dot(normal) * normal
}
