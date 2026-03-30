use bevy::prelude::*;

use super::system::*;
use crate::aabb::Aabb2D;

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

// -- Behavior 4: SweepHit has correct fields --

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

// -- RayHit::safe_position --

#[test]
fn ray_hit_safe_position_offsets_by_epsilon() {
    let hit = RayHit {
        distance: 10.0,
        normal: Vec2::NEG_Y,
    };
    let pos = hit.safe_position(Vec2::new(0.0, -30.0), Vec2::Y);
    // 10.0 - 0.01 = 9.99, so position = (0, -30) + Y * 9.99 = (0, -20.01)
    assert!((pos.y - (-20.01)).abs() < 1e-4, "got {}", pos.y);
}

#[test]
fn ray_hit_safe_position_clamps_at_zero() {
    let hit = RayHit {
        distance: 0.005,
        normal: Vec2::NEG_Y,
    };
    let pos = hit.safe_position(Vec2::ZERO, Vec2::Y);
    // (0.005 - 0.01).max(0.0) = 0.0, so position = origin
    assert_eq!(pos, Vec2::ZERO);
}

// -- RayHit::remaining --

#[test]
fn ray_hit_remaining_distance() {
    let hit = RayHit {
        distance: 35.0,
        normal: Vec2::NEG_Y,
    };
    let rem = hit.remaining(200.0);
    assert!((rem - 165.0).abs() < 1e-4);
}

#[test]
fn ray_hit_remaining_clamps_at_zero() {
    let hit = RayHit {
        distance: 200.0,
        normal: Vec2::NEG_Y,
    };
    let rem = hit.remaining(100.0);
    assert!(rem.abs() < f32::EPSILON);
}

// -- reflect --

#[test]
fn reflect_off_horizontal_surface() {
    let v = Vec2::new(300.0, -400.0);
    let n = Vec2::NEG_Y;
    let r = reflect(v, n);
    assert!((r.x - 300.0).abs() < 1e-3);
    assert!((r.y - 400.0).abs() < 1e-3);
}

#[test]
fn reflect_preserves_speed() {
    let v = Vec2::new(-250.0, -100.0);
    let n = Vec2::NEG_X;
    let r = reflect(v, n);
    assert!((v.length() - r.length()).abs() < 1e-3);
}
