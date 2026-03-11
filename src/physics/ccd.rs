//! Continuous collision detection helpers shared across physics systems.

use bevy::prelude::*;

/// Maximum number of bounces resolved per bolt per frame.
///
/// Prevents infinite loops in degenerate geometries.
pub const MAX_BOUNCES: u32 = 4;

/// Sub-pixel separation gap applied after each collision.
///
/// The bolt is placed this far outside the target's expanded AABB to prevent
/// floating-point touching on the next sweep.
pub const CCD_EPSILON: f32 = 0.01;

/// Result of a ray-vs-expanded-AABB intersection test.
pub struct RayHit {
    /// Distance along the ray to the entry point.
    pub distance: f32,
    /// Outward face normal at the entry point.
    pub normal: Vec2,
}

/// Casts a ray against an AABB and returns the entry distance and face normal.
///
/// The AABB should already be Minkowski-expanded by the bolt radius so that
/// a point-ray test is equivalent to a circle-vs-rectangle test.
///
/// Returns `None` if the ray misses, the origin is inside the AABB, or the
/// hit is beyond `max_dist`.
pub fn ray_vs_aabb(
    origin: Vec2,
    direction: Vec2,
    max_dist: f32,
    aabb_center: Vec2,
    aabb_half_extents: Vec2,
) -> Option<RayHit> {
    let aabb_min = aabb_center - aabb_half_extents;
    let aabb_max = aabb_center + aabb_half_extents;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hit_from_below() {
        let hit = ray_vs_aabb(
            Vec2::new(0.0, -30.0),
            Vec2::Y,
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        )
        .expect("should hit");

        assert!(
            (hit.distance - 10.0).abs() < 0.01,
            "distance={}",
            hit.distance
        );
        assert_eq!(hit.normal, Vec2::NEG_Y);
    }

    #[test]
    fn ray_hit_from_side() {
        let hit = ray_vs_aabb(
            Vec2::new(-60.0, 0.0),
            Vec2::X,
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        )
        .expect("should hit");

        assert!(
            (hit.distance - 17.0).abs() < 0.01,
            "distance={}",
            hit.distance
        );
        assert_eq!(hit.normal, Vec2::NEG_X);
    }

    #[test]
    fn ray_miss_parallel() {
        let result = ray_vs_aabb(
            Vec2::new(0.0, -30.0),
            Vec2::X,
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        );
        assert!(result.is_none(), "parallel ray should miss");
    }

    #[test]
    fn ray_miss_beyond_max_dist() {
        let result = ray_vs_aabb(
            Vec2::new(0.0, -200.0),
            Vec2::Y,
            10.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        );
        assert!(result.is_none(), "ray should not reach cell");
    }

    #[test]
    fn ray_origin_inside_returns_none() {
        let result = ray_vs_aabb(
            Vec2::ZERO,
            Vec2::Y,
            100.0,
            Vec2::ZERO,
            Vec2::new(43.0, 20.0),
        );
        assert!(result.is_none(), "origin inside AABB should return None");
    }
}
