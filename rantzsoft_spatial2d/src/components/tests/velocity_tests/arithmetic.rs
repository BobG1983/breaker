use bevy::prelude::*;

use super::super::super::definitions::*;

#[test]
fn velocity_add_vec2() {
    let result = Velocity2D(Vec2::new(1.0, 2.0)) + Vec2::new(3.0, 4.0);
    assert_eq!(result, Velocity2D(Vec2::new(4.0, 6.0)));
}

#[test]
fn velocity_sub_vec2() {
    let result = Velocity2D(Vec2::new(5.0, 5.0)) - Vec2::new(1.0, 2.0);
    assert_eq!(result, Velocity2D(Vec2::new(4.0, 3.0)));
}

#[test]
fn velocity_mul_f32() {
    let result = Velocity2D(Vec2::new(2.0, 3.0)) * 2.0;
    assert_eq!(result, Velocity2D(Vec2::new(4.0, 6.0)));
}

#[test]
fn velocity_div_f32() {
    let result = Velocity2D(Vec2::new(6.0, 8.0)) / 2.0;
    assert_eq!(result, Velocity2D(Vec2::new(3.0, 4.0)));
}

// ── PreviousVelocity ────────────────────────────────────────

#[test]
fn previous_velocity_default_is_zero() {
    assert_eq!(PreviousVelocity::default().0, Vec2::ZERO);
}
