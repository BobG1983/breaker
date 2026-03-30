use bevy::prelude::*;

use super::super::definitions::*;

// ── ApplyVelocity ────────────────────────────────────────────

#[test]
fn apply_velocity_marker_is_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
    let entity = app.world_mut().spawn(ApplyVelocity).id();
    assert!(app.world().get::<ApplyVelocity>(entity).is_some());
}

// ── Velocity2D ─────────────────────────────────────────────

#[test]
fn velocity_default_is_zero() {
    assert_eq!(Velocity2D::default().0, Vec2::ZERO);
}

#[test]
fn velocity_speed_returns_magnitude() {
    assert!(
        (Velocity2D(Vec2::new(3.0, 4.0)).speed() - 5.0).abs() < f32::EPSILON,
        "speed of (3, 4) should be 5.0"
    );
}

#[test]
fn velocity_speed_zero_returns_zero() {
    assert!(
        Velocity2D(Vec2::ZERO).speed().abs() < f32::EPSILON,
        "speed of zero velocity should be 0.0"
    );
}

#[test]
fn velocity_clamped_high_to_max() {
    let v = Velocity2D(Vec2::new(0.0, 800.0)).clamped(200.0, 600.0);
    let speed = v.0.length();
    assert!(
        (speed - 600.0).abs() < 1e-3,
        "magnitude should be clamped to 600.0, got {speed}"
    );
    // Direction should be preserved (pointing up).
    let dir = v.0.normalize();
    assert!(
        (dir.y - 1.0).abs() < 1e-5,
        "direction should be (0, 1), got {dir:?}"
    );
}

#[test]
fn velocity_clamped_exactly_at_max_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 600.0)).clamped(200.0, 600.0);
    assert!(
        (v.0.length() - 600.0).abs() < 1e-3,
        "velocity exactly at max should remain unchanged"
    );
}

#[test]
fn velocity_clamped_low_to_min() {
    let v = Velocity2D(Vec2::new(0.0, 100.0)).clamped(200.0, 600.0);
    let speed = v.0.length();
    assert!(
        (speed - 200.0).abs() < 1e-3,
        "magnitude should be clamped up to 200.0, got {speed}"
    );
    let dir = v.0.normalize();
    assert!(
        (dir.y - 1.0).abs() < 1e-5,
        "direction should be preserved as (0, 1), got {dir:?}"
    );
}

#[test]
fn velocity_clamped_exactly_at_min_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 200.0)).clamped(200.0, 600.0);
    assert!(
        (v.0.length() - 200.0).abs() < 1e-3,
        "velocity exactly at min should remain unchanged"
    );
}

#[test]
fn velocity_clamped_zero_returns_zero() {
    let v = Velocity2D(Vec2::ZERO).clamped(200.0, 600.0);
    assert_eq!(v, Velocity2D(Vec2::ZERO), "zero velocity should stay zero");
}

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

// ── Velocity2D::clamped() preserves direction for diagonal velocity ──

#[test]
fn velocity_clamped_preserves_direction_diagonal() {
    // Positive diagonal: magnitude 500.0, direction (0.6, 0.8)
    let v = Velocity2D(Vec2::new(300.0, 400.0));
    let result = v.clamped(200.0, 400.0);

    let speed = result.0.length();
    assert!(
        (speed - 400.0).abs() < 1e-3,
        "magnitude should be clamped to 400.0, got {speed}"
    );

    let dir = result.0.normalize();
    assert!(
        (dir.x - 0.6).abs() < 1e-5,
        "direction x should be 0.6, got {}",
        dir.x
    );
    assert!(
        (dir.y - 0.8).abs() < 1e-5,
        "direction y should be 0.8, got {}",
        dir.y
    );

    // Edge case: negative diagonal, same magnitude 500.0, clamped to 400.0
    let v_neg = Velocity2D(Vec2::new(-300.0, -400.0));
    let result_neg = v_neg.clamped(200.0, 400.0);

    let speed_neg = result_neg.0.length();
    assert!(
        (speed_neg - 400.0).abs() < 1e-3,
        "negative diagonal magnitude should be clamped to 400.0, got {speed_neg}"
    );

    let dir_neg = result_neg.0.normalize();
    assert!(
        (dir_neg.x - (-0.6)).abs() < 1e-5,
        "negative direction x should be -0.6, got {}",
        dir_neg.x
    );
    assert!(
        (dir_neg.y - (-0.8)).abs() < 1e-5,
        "negative direction y should be -0.8, got {}",
        dir_neg.y
    );
}
