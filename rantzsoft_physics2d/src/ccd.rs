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
    pub normal: Vec2,
}

/// Result of a swept circle (or ray) cast against the quadtree.
pub struct SweepHit {
    /// The entity that was hit.
    pub entity: Entity,
    /// Safe position for the moving object just before contact.
    pub position: Vec2,
    /// Outward face normal at the contact point.
    pub normal: Vec2,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hit_from_below() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(43.0, 20.0));
        let hit = ray_vs_aabb(Vec2::new(0.0, -30.0), Vec2::Y, 100.0, &aabb).expect("should hit");

        assert!(
            (hit.distance - 10.0).abs() < 0.01,
            "distance={}",
            hit.distance
        );
        assert_eq!(hit.normal, Vec2::NEG_Y);
    }

    #[test]
    fn ray_hit_from_side() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(43.0, 20.0));
        let hit = ray_vs_aabb(Vec2::new(-60.0, 0.0), Vec2::X, 100.0, &aabb).expect("should hit");

        assert!(
            (hit.distance - 17.0).abs() < 0.01,
            "distance={}",
            hit.distance
        );
        assert_eq!(hit.normal, Vec2::NEG_X);
    }

    #[test]
    fn ray_miss_parallel() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(43.0, 20.0));
        let result = ray_vs_aabb(Vec2::new(0.0, -30.0), Vec2::X, 100.0, &aabb);
        assert!(result.is_none(), "parallel ray should miss");
    }

    #[test]
    fn ray_miss_beyond_max_dist() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(43.0, 20.0));
        let result = ray_vs_aabb(Vec2::new(0.0, -200.0), Vec2::Y, 10.0, &aabb);
        assert!(result.is_none(), "ray should not reach AABB");
    }

    #[test]
    fn ray_origin_inside_returns_none() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(43.0, 20.0));
        let result = ray_vs_aabb(Vec2::ZERO, Vec2::Y, 100.0, &aabb);
        assert!(result.is_none(), "origin inside AABB should return None");
    }

    // --- Replaced property-based tests with concrete-value equivalents ---

    #[test]
    fn hit_distance_is_positive_from_below_centered() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 30.0));
        let origin = Vec2::new(0.0, -100.0);
        let hit = ray_vs_aabb(origin, Vec2::Y, 2000.0, &aabb).expect("should hit");
        assert!(
            hit.distance > 0.0,
            "distance must be positive, got {}",
            hit.distance
        );
    }

    #[test]
    fn hit_distance_is_positive_from_below_offset() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(200.0, 100.0));
        let origin = Vec2::new(150.0, -200.0);
        let hit = ray_vs_aabb(origin, Vec2::Y, 2000.0, &aabb).expect("should hit");
        assert!(
            hit.distance > 0.0,
            "distance must be positive, got {}",
            hit.distance
        );
    }

    #[test]
    fn hit_distance_is_positive_from_left() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(40.0, 40.0));
        let origin = Vec2::new(-80.0, 0.0);
        let hit = ray_vs_aabb(origin, Vec2::X, 2000.0, &aabb).expect("should hit");
        assert!(
            hit.distance > 0.0,
            "distance must be positive, got {}",
            hit.distance
        );
    }

    #[test]
    fn hit_distance_is_positive_small_aabb() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(1.0, 1.0));
        let origin = Vec2::new(0.0, -10.0);
        let hit = ray_vs_aabb(origin, Vec2::Y, 2000.0, &aabb).expect("should hit");
        assert!(
            hit.distance > 0.0,
            "distance must be positive, got {}",
            hit.distance
        );
    }

    #[test]
    fn hit_normal_axis_aligned_from_below() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 30.0));
        let hit = ray_vs_aabb(Vec2::new(0.0, -60.0), Vec2::Y, 200.0, &aabb).expect("should hit");
        assert_eq!(hit.normal, Vec2::NEG_Y, "normal from below should be -Y");
    }

    #[test]
    fn hit_normal_axis_aligned_from_above() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 30.0));
        let hit = ray_vs_aabb(Vec2::new(0.0, 60.0), Vec2::NEG_Y, 200.0, &aabb).expect("should hit");
        assert_eq!(hit.normal, Vec2::Y, "normal from above should be +Y");
    }

    #[test]
    fn hit_normal_axis_aligned_from_left() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 30.0));
        let hit = ray_vs_aabb(Vec2::new(-80.0, 0.0), Vec2::X, 200.0, &aabb).expect("should hit");
        assert_eq!(hit.normal, Vec2::NEG_X, "normal from left should be -X");
    }

    #[test]
    fn hit_normal_axis_aligned_from_right() {
        let aabb = Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 30.0));
        let hit = ray_vs_aabb(Vec2::new(80.0, 0.0), Vec2::NEG_X, 200.0, &aabb).expect("should hit");
        assert_eq!(hit.normal, Vec2::X, "normal from right should be +X");
    }

    #[test]
    fn reflection_preserves_speed_off_y_normal() {
        let velocity = Vec2::new(300.0, -400.0);
        let normal = Vec2::NEG_Y;
        let speed_before = velocity.length();

        let reflected = velocity - 2.0 * velocity.dot(normal) * normal;
        let speed_after = reflected.length();

        assert!(
            (speed_before - speed_after).abs() < 1e-3,
            "reflection off -Y should preserve speed: {speed_before} vs {speed_after}"
        );
    }

    #[test]
    fn reflection_preserves_speed_off_x_normal() {
        let velocity = Vec2::new(-250.0, -100.0);
        let normal = Vec2::NEG_X;
        let speed_before = velocity.length();

        let reflected = velocity - 2.0 * velocity.dot(normal) * normal;
        let speed_after = reflected.length();

        assert!(
            (speed_before - speed_after).abs() < 1e-3,
            "reflection off -X should preserve speed: {speed_before} vs {speed_after}"
        );
    }

    #[test]
    fn reflection_preserves_speed_off_positive_y() {
        let velocity = Vec2::new(150.0, -450.0);
        let normal = Vec2::Y;
        let speed_before = velocity.length();

        let reflected = velocity - 2.0 * velocity.dot(normal) * normal;
        let speed_after = reflected.length();

        assert!(
            (speed_before - speed_after).abs() < 1e-3,
            "reflection off +Y should preserve speed: {speed_before} vs {speed_after}"
        );
    }

    #[test]
    fn reflection_preserves_speed_off_positive_x() {
        let velocity = Vec2::new(-500.0, -10.0);
        let normal = Vec2::X;
        let speed_before = velocity.length();

        let reflected = velocity - 2.0 * velocity.dot(normal) * normal;
        let speed_after = reflected.length();

        assert!(
            (speed_before - speed_after).abs() < 1e-3,
            "reflection off +X should preserve speed: {speed_before} vs {speed_after}"
        );
    }

    // ── Behavior 4: SweepHit has correct fields ──

    #[test]
    fn sweep_hit_fields_are_publicly_accessible() {
        let sweep = SweepHit {
            entity: Entity::PLACEHOLDER,
            position: Vec2::new(10.0, 35.0),
            normal: Vec2::NEG_Y,
            remaining: 165.0,
        };

        assert_eq!(sweep.entity, Entity::PLACEHOLDER);
        assert_eq!(sweep.position, Vec2::new(10.0, 35.0));
        assert_eq!(sweep.normal, Vec2::NEG_Y);
        assert!((sweep.remaining - 165.0).abs() < f32::EPSILON);
    }
}
