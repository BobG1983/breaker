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

#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    use super::*;

    /// Generate a positive, finite float in a useful range.
    fn positive_float() -> impl Strategy<Value = f32> {
        1.0_f32..1000.0
    }

    /// Generate a finite float in a useful range.
    fn bounded_float() -> impl Strategy<Value = f32> {
        -500.0_f32..500.0
    }

    proptest! {
        /// A hit distance must always be positive (ray starts outside AABB).
        #[test]
        fn hit_distance_is_positive(
            ox in bounded_float(),
            oy in -500.0_f32..-50.0,
            hw in positive_float(),
            hh in positive_float(),
        ) {
            let origin = Vec2::new(ox.clamp(-hw + 1.0, hw - 1.0), oy - hh);
            let direction = Vec2::Y;
            let max_dist = 2000.0;
            let center = Vec2::ZERO;
            let half = Vec2::new(hw, hh);

            if let Some(hit) = ray_vs_aabb(origin, direction, max_dist, center, half) {
                prop_assert!(hit.distance > 0.0, "hit distance must be positive, got {}", hit.distance);
            }
        }

        /// Hit normal is always axis-aligned and unit length.
        #[test]
        fn hit_normal_is_unit_axis_aligned(
            ox in bounded_float(),
            oy in -500.0_f32..-50.0,
            hw in positive_float(),
            hh in positive_float(),
        ) {
            let origin = Vec2::new(ox.clamp(-hw + 1.0, hw - 1.0), oy - hh);
            let direction = Vec2::Y;
            let max_dist = 2000.0;
            let center = Vec2::ZERO;
            let half = Vec2::new(hw, hh);

            if let Some(hit) = ray_vs_aabb(origin, direction, max_dist, center, half) {
                let len = hit.normal.length();
                prop_assert!(
                    (len - 1.0).abs() < 1e-5,
                    "normal should be unit length, got {len}"
                );
                // Must be axis-aligned: one component zero, one +-1
                let is_axis = (hit.normal.x.abs() < f32::EPSILON && (hit.normal.y.abs() - 1.0).abs() < f32::EPSILON)
                    || (hit.normal.y.abs() < f32::EPSILON && (hit.normal.x.abs() - 1.0).abs() < f32::EPSILON);
                prop_assert!(is_axis, "normal must be axis-aligned, got {:?}", hit.normal);
            }
        }

        /// Reflection off an AABB surface preserves speed (magnitude).
        #[test]
        fn reflection_preserves_speed(
            vx in -500.0_f32..500.0,
            vy in -500.0_f32..-10.0,
            nx in prop_oneof![Just(0.0_f32), Just(1.0_f32), Just(-1.0_f32)],
            ny in prop_oneof![Just(0.0_f32), Just(1.0_f32), Just(-1.0_f32)],
        ) {
            let normal = Vec2::new(nx, ny);
            if normal.length() < 0.5 {
                return Ok(());
            }
            let normal = normal.normalize();
            let velocity = Vec2::new(vx, vy);
            let speed_before = velocity.length();

            let reflected = velocity - 2.0 * velocity.dot(normal) * normal;
            let speed_after = reflected.length();

            prop_assert!(
                (speed_before - speed_after).abs() < 1e-3,
                "reflection should preserve speed: {speed_before} vs {speed_after}"
            );
        }
    }
}
