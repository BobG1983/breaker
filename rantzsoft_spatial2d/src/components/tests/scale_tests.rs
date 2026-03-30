use bevy::prelude::*;

use super::super::definitions::*;

// ── Scale2D ─────────────────────────────────────────────────

#[test]
fn scale_default_is_uniform_one() {
    let s = Scale2D::default();
    assert!((s.x - 1.0).abs() < f32::EPSILON);
    assert!((s.y - 1.0).abs() < f32::EPSILON);
}

#[test]
fn scale_new_stores_values() {
    let s = Scale2D::new(2.0, 3.0);
    assert!((s.x - 2.0).abs() < f32::EPSILON);
    assert!((s.y - 3.0).abs() < f32::EPSILON);
}

#[test]
#[should_panic(expected = "Scale2D components must be non-zero")]
fn scale_new_panics_on_zero_x() {
    let _ = Scale2D::new(0.0, 1.0);
}

#[test]
#[should_panic(expected = "Scale2D components must be non-zero")]
fn scale_new_panics_on_zero_y() {
    let _ = Scale2D::new(1.0, 0.0);
}

#[test]
fn scale_uniform() {
    let s = Scale2D::uniform(2.0);
    assert_eq!(s, Scale2D { x: 2.0, y: 2.0 });
}

#[test]
fn scale_to_vec3() {
    let v = Scale2D::new(2.0, 3.0).to_vec3();
    assert_eq!(v, Vec3::new(2.0, 3.0, 1.0));
}

// ── GlobalScale2D ───────────────────────────────────────────

#[test]
fn global_scale_default_is_one_one() {
    let s = GlobalScale2D::default();
    assert!(
        (s.x - 1.0).abs() < f32::EPSILON,
        "GlobalScale2D.x should default to 1.0, got {}",
        s.x
    );
    assert!(
        (s.y - 1.0).abs() < f32::EPSILON,
        "GlobalScale2D.y should default to 1.0, got {}",
        s.y
    );
}
