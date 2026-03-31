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

// ── Velocity2D::clamp_angle() ─────────────────────────────────

#[test]
fn velocity_clamp_angle_shallow_from_horizontal_clamped_toward_vertical() {
    // Angle from horizontal ~7.1 deg (0.1244 rad), min bound 15 deg (0.2618 rad)
    let v = Velocity2D(Vec2::new(400.0, 50.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((0.2618, 0.2618));

    // Speed must be preserved
    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );

    // Angle from horizontal should be clamped to 0.2618 rad
    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 0.2618).abs() < 1e-3,
        "angle from horizontal should be 0.2618 rad, got {result_angle}"
    );

    // Both components should remain positive
    assert!(
        result.0.x > 0.0,
        "x should remain positive, got {}",
        result.0.x
    );
    assert!(
        result.0.y > 0.0,
        "y should remain positive, got {}",
        result.0.y
    );

    // Check approximate values
    let expected_x = original_speed * 0.2618_f32.cos();
    let expected_y = original_speed * 0.2618_f32.sin();
    assert!(
        (result.0.x - expected_x).abs() < 1e-1,
        "x should be ~{expected_x}, got {}",
        result.0.x
    );
    assert!(
        (result.0.y - expected_y).abs() < 1e-1,
        "y should be ~{expected_y}, got {}",
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_steep_near_vertical_clamped_toward_horizontal() {
    // Angle from horizontal ~82.9 deg (1.4464 rad), upper bound PI/2 - 0.2618 = 1.3090 rad
    let v = Velocity2D(Vec2::new(50.0, 400.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((0.2618, 0.2618));

    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );

    let upper_bound = std::f32::consts::FRAC_PI_2 - 0.2618;
    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - upper_bound).abs() < 1e-3,
        "angle from horizontal should be {upper_bound} rad, got {result_angle}"
    );

    assert!(
        result.0.x > 0.0,
        "x should remain positive, got {}",
        result.0.x
    );
    assert!(
        result.0.y > 0.0,
        "y should remain positive, got {}",
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_within_bounds_stays_unchanged() {
    // Angle from horizontal is 45 degrees (PI/4), within [0.2618, 1.3090]
    let v = Velocity2D(Vec2::new(300.0, 300.0));
    let result = v.clamp_angle((0.2618, 0.2618));

    assert_eq!(
        result, v,
        "velocity within angle bounds should be unchanged"
    );
}

#[test]
fn velocity_clamp_angle_exactly_at_lower_bound_unchanged() {
    // Construct a velocity exactly at 0.2618 rad from horizontal
    let speed = 403.11_f32;
    let angle = 0.2618_f32;
    let v = Velocity2D(Vec2::new(speed * angle.cos(), speed * angle.sin()));
    let result = v.clamp_angle((0.2618, 0.2618));

    assert!(
        (result.0.x - v.0.x).abs() < 1e-3,
        "x should be unchanged at lower bound, expected {}, got {}",
        v.0.x,
        result.0.x
    );
    assert!(
        (result.0.y - v.0.y).abs() < 1e-3,
        "y should be unchanged at lower bound, expected {}, got {}",
        v.0.y,
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_exactly_at_upper_bound_unchanged() {
    // Construct a velocity exactly at upper bound (PI/2 - 0.2618) from horizontal
    let speed = 403.11_f32;
    let upper = std::f32::consts::FRAC_PI_2 - 0.2618;
    let v = Velocity2D(Vec2::new(speed * upper.cos(), speed * upper.sin()));
    let result = v.clamp_angle((0.2618, 0.2618));

    assert!(
        (result.0.x - v.0.x).abs() < 1e-3,
        "x should be unchanged at upper bound, expected {}, got {}",
        v.0.x,
        result.0.x
    );
    assert!(
        (result.0.y - v.0.y).abs() < 1e-3,
        "y should be unchanged at upper bound, expected {}, got {}",
        v.0.y,
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_preserves_speed_at_high_velocity() {
    let v = Velocity2D(Vec2::new(4000.0, 500.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((0.2618, 0.2618));

    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );
}

#[test]
fn velocity_clamp_angle_preserves_signs_quadrant_pos_pos() {
    // Quadrant (+, +): both positive, angle too shallow
    let v = Velocity2D(Vec2::new(400.0, 50.0));
    let result = v.clamp_angle((0.2618, 0.2618));

    assert!(result.0.x > 0.0, "x should be positive, got {}", result.0.x);
    assert!(result.0.y > 0.0, "y should be positive, got {}", result.0.y);

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 0.2618).abs() < 1e-3,
        "angle from horizontal should be 0.2618 rad, got {result_angle}"
    );
}

#[test]
fn velocity_clamp_angle_preserves_signs_quadrant_neg_pos() {
    // Quadrant (-, +): x negative, y positive, angle too shallow
    let v = Velocity2D(Vec2::new(-400.0, 50.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((0.2618, 0.2618));

    assert!(result.0.x < 0.0, "x should be negative, got {}", result.0.x);
    assert!(result.0.y > 0.0, "y should be positive, got {}", result.0.y);

    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 0.2618).abs() < 1e-3,
        "angle from horizontal should be 0.2618 rad, got {result_angle}"
    );
}

#[test]
fn velocity_clamp_angle_preserves_signs_quadrant_neg_neg() {
    // Quadrant (-, -): both negative, angle too shallow
    let v = Velocity2D(Vec2::new(-400.0, -50.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((0.2618, 0.2618));

    assert!(result.0.x < 0.0, "x should be negative, got {}", result.0.x);
    assert!(result.0.y < 0.0, "y should be negative, got {}", result.0.y);

    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 0.2618).abs() < 1e-3,
        "angle from horizontal should be 0.2618 rad, got {result_angle}"
    );
}

#[test]
fn velocity_clamp_angle_preserves_signs_quadrant_pos_neg() {
    // Quadrant (+, -): x positive, y negative, angle too shallow
    let v = Velocity2D(Vec2::new(400.0, -50.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((0.2618, 0.2618));

    assert!(result.0.x > 0.0, "x should be positive, got {}", result.0.x);
    assert!(result.0.y < 0.0, "y should be negative, got {}", result.0.y);

    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 0.2618).abs() < 1e-3,
        "angle from horizontal should be 0.2618 rad, got {result_angle}"
    );
}

#[test]
fn velocity_clamp_angle_zero_velocity_returns_zero() {
    let v = Velocity2D(Vec2::ZERO);
    let result = v.clamp_angle((0.2618, 0.2618));
    assert_eq!(
        result,
        Velocity2D(Vec2::ZERO),
        "zero velocity should remain zero"
    );
}

#[test]
fn velocity_clamp_angle_near_zero_velocity_returned_unchanged() {
    // Extremely small magnitude — below EPSILON threshold, treated as zero
    let v = Velocity2D(Vec2::new(1e-10, 1e-10));
    let result = v.clamp_angle((0.2618, 0.2618));
    assert_eq!(result, v, "near-zero velocity should be returned unchanged");
}

#[test]
fn velocity_clamp_angle_purely_horizontal_positive_x_clamps_to_min() {
    // Angle from horizontal is 0.0 rad, below lower bound
    let v = Velocity2D(Vec2::new(400.0, 0.0));
    let result = v.clamp_angle((0.2618, 0.2618));

    let result_speed = result.speed();
    assert!(
        (result_speed - 400.0).abs() < 1e-3,
        "speed should be preserved at 400.0, got {result_speed}"
    );

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 0.2618).abs() < 1e-3,
        "angle from horizontal should be 0.2618 rad, got {result_angle}"
    );

    // y=0 zero guard defaults sign_y to 1.0, so y should be positive
    assert!(
        result.0.y > 0.0,
        "y should be positive (zero guard defaults to 1.0), got {}",
        result.0.y
    );
    assert!(
        result.0.x > 0.0,
        "x should remain positive, got {}",
        result.0.x
    );

    let expected_x = 400.0 * 0.2618_f32.cos();
    let expected_y = 400.0 * 0.2618_f32.sin();
    assert!(
        (result.0.x - expected_x).abs() < 1e-1,
        "x should be ~{expected_x}, got {}",
        result.0.x
    );
    assert!(
        (result.0.y - expected_y).abs() < 1e-1,
        "y should be ~{expected_y}, got {}",
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_purely_vertical_clamps_to_max() {
    // Angle from horizontal is PI/2, above upper bound
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.clamp_angle((0.2618, 0.2618));

    let result_speed = result.speed();
    assert!(
        (result_speed - 400.0).abs() < 1e-3,
        "speed should be preserved at 400.0, got {result_speed}"
    );

    let upper_bound = std::f32::consts::FRAC_PI_2 - 0.2618;
    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - upper_bound).abs() < 1e-3,
        "angle from horizontal should be {upper_bound} rad, got {result_angle}"
    );

    // x=0 zero guard defaults sign_x to 1.0, so x should be positive
    assert!(
        result.0.x > 0.0,
        "x should be positive (zero guard defaults to 1.0), got {}",
        result.0.x
    );
    assert!(
        result.0.y > 0.0,
        "y should remain positive, got {}",
        result.0.y
    );

    let expected_x = 400.0 * upper_bound.cos();
    let expected_y = 400.0 * upper_bound.sin();
    assert!(
        (result.0.x - expected_x).abs() < 1e-1,
        "x should be ~{expected_x}, got {}",
        result.0.x
    );
    assert!(
        (result.0.y - expected_y).abs() < 1e-1,
        "y should be ~{expected_y}, got {}",
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_purely_horizontal_negative_x_clamps_to_min() {
    // Angle from horizontal is 0.0 rad, negative x direction
    let v = Velocity2D(Vec2::new(-400.0, 0.0));
    let result = v.clamp_angle((0.2618, 0.2618));

    let result_speed = result.speed();
    assert!(
        (result_speed - 400.0).abs() < 1e-3,
        "speed should be preserved at 400.0, got {result_speed}"
    );

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 0.2618).abs() < 1e-3,
        "angle from horizontal should be 0.2618 rad, got {result_angle}"
    );

    // sign_x = -1.0 (negative x preserved), sign_y = 1.0 (y=0 zero guard defaults to 1.0)
    assert!(
        result.0.x < 0.0,
        "x should remain negative, got {}",
        result.0.x
    );
    assert!(
        result.0.y > 0.0,
        "y should be positive (zero guard defaults to 1.0), got {}",
        result.0.y
    );

    let expected_x = -400.0 * 0.2618_f32.cos();
    let expected_y = 400.0 * 0.2618_f32.sin();
    assert!(
        (result.0.x - expected_x).abs() < 1e-1,
        "x should be ~{expected_x}, got {}",
        result.0.x
    );
    assert!(
        (result.0.y - expected_y).abs() < 1e-1,
        "y should be ~{expected_y}, got {}",
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_overlapping_bounds_collapses_to_single_angle() {
    // bounds.0=1.0, bounds.1=1.0 => upper = PI/2 - 1.0 ≈ 0.5708 which is < 1.0
    // upper gets clamped to bounds.0=1.0, so valid range is [1.0, 1.0]
    let v = Velocity2D(Vec2::new(300.0, 300.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((1.0, 1.0));

    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 1.0).abs() < 1e-3,
        "angle should be clamped to 1.0 rad, got {result_angle}"
    );

    assert!(
        result.0.x > 0.0,
        "x should remain positive, got {}",
        result.0.x
    );
    assert!(
        result.0.y > 0.0,
        "y should remain positive, got {}",
        result.0.y
    );
}

#[test]
fn velocity_clamp_angle_extreme_overlap_collapses_to_lower_bound() {
    // bounds.0=1.5, bounds.1=1.5 => upper = PI/2 - 1.5 ≈ 0.0708, clamped to 1.5
    // Range collapses to [1.5, 1.5]
    let v = Velocity2D(Vec2::new(300.0, 300.0));
    let original_speed = v.speed();
    let result = v.clamp_angle((1.5, 1.5));

    let result_speed = result.speed();
    assert!(
        (result_speed - original_speed).abs() < 1e-3,
        "speed should be preserved at {original_speed}, got {result_speed}"
    );

    let result_angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (result_angle - 1.5).abs() < 1e-3,
        "angle should be clamped to 1.5 rad, got {result_angle}"
    );
}

// ── Velocity2D::clamp() ──────────────────────────────────────

#[test]
fn velocity_clamp_high_speed_clamped_to_max() {
    let v = Velocity2D(Vec2::new(0.0, 800.0));
    let result = v.clamp(200.0, 600.0);
    let speed = result.speed();
    assert!(
        (speed - 600.0).abs() < 1e-3,
        "speed should be clamped to max 600.0, got {speed}"
    );
}

#[test]
fn velocity_clamp_low_speed_clamped_to_min() {
    let v = Velocity2D(Vec2::new(0.0, 100.0));
    let result = v.clamp(200.0, 600.0);
    let speed = result.speed();
    assert!(
        (speed - 200.0).abs() < 1e-3,
        "speed should be clamped to min 200.0, got {speed}"
    );
}

#[test]
fn velocity_clamp_within_bounds_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "velocity within bounds should be unchanged");
}

#[test]
fn velocity_clamp_zero_returns_zero() {
    let v = Velocity2D(Vec2::ZERO);
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "zero velocity should stay zero");
}

#[test]
fn velocity_clamp_preserves_direction() {
    let v = Velocity2D(Vec2::new(300.0, 400.0)); // speed 500, direction (0.6, 0.8)
    let result = v.clamp(200.0, 400.0);
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
}

// ── Velocity2D::with_speed() ────────────────────────────────

#[test]
fn velocity_with_speed_sets_magnitude() {
    let v = Velocity2D(Vec2::new(3.0, 4.0)); // speed 5
    let result = v.with_speed(10.0);
    assert!(
        (result.speed() - 10.0).abs() < 1e-3,
        "speed should be 10.0, got {}",
        result.speed()
    );
}

#[test]
fn velocity_with_speed_preserves_direction() {
    let v = Velocity2D(Vec2::new(3.0, 4.0));
    let result = v.with_speed(10.0);
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
}

#[test]
fn velocity_with_speed_zero_velocity_returns_zero() {
    let v = Velocity2D(Vec2::ZERO);
    let result = v.with_speed(500.0);
    assert_eq!(result, v, "zero velocity with_speed should stay zero");
}

#[test]
fn velocity_with_speed_preserves_negative_direction() {
    let v = Velocity2D(Vec2::new(-3.0, -4.0)); // speed 5, direction (-0.6, -0.8)
    let result = v.with_speed(10.0);
    assert!(
        (result.speed() - 10.0).abs() < 1e-3,
        "speed should be 10.0, got {}",
        result.speed()
    );
    let dir = result.0.normalize();
    assert!(
        (dir.x - (-0.6)).abs() < 1e-5,
        "direction x should be -0.6, got {}",
        dir.x
    );
    assert!(
        (dir.y - (-0.8)).abs() < 1e-5,
        "direction y should be -0.8, got {}",
        dir.y
    );
}

#[test]
fn velocity_with_speed_zero_produces_zero_vector() {
    let v = Velocity2D(Vec2::new(3.0, 4.0));
    let result = v.with_speed(0.0);
    assert!(
        result.speed() < f32::EPSILON,
        "with_speed(0) should produce zero magnitude, got {}",
        result.speed()
    );
}

#[test]
fn velocity_clamp_exactly_at_min_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 200.0));
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "velocity exactly at min should be unchanged");
}

#[test]
fn velocity_clamp_exactly_at_max_unchanged() {
    let v = Velocity2D(Vec2::new(0.0, 600.0));
    let result = v.clamp(200.0, 600.0);
    assert_eq!(result, v, "velocity exactly at max should be unchanged");
}

#[test]
fn velocity_clamp_preserves_negative_direction() {
    let v = Velocity2D(Vec2::new(-300.0, -400.0)); // speed 500
    let result = v.clamp(200.0, 400.0);
    assert!(
        (result.speed() - 400.0).abs() < 1e-3,
        "speed should be clamped to 400.0, got {}",
        result.speed()
    );
    let dir = result.0.normalize();
    assert!(
        (dir.x - (-0.6)).abs() < 1e-5,
        "direction x should be -0.6, got {}",
        dir.x
    );
    assert!(
        (dir.y - (-0.8)).abs() < 1e-5,
        "direction y should be -0.8, got {}",
        dir.y
    );
}

// ── Velocity2D::from_angle_up() ─────────────────────────────

#[test]
fn from_angle_up_zero_is_straight_up() {
    let v = Velocity2D::from_angle_up(0.0, 400.0);
    assert!((v.0.x).abs() < 1e-3, "x should be ~0, got {}", v.0.x);
    assert!(
        (v.0.y - 400.0).abs() < 1e-3,
        "y should be 400, got {}",
        v.0.y
    );
}

#[test]
fn from_angle_up_positive_is_clockwise_rightward() {
    let v = Velocity2D::from_angle_up(std::f32::consts::FRAC_PI_4, 400.0);
    assert!(v.0.x > 0.0, "positive angle should have positive x");
    assert!(v.0.y > 0.0, "PI/4 from up should still have positive y");
    assert!(
        (v.speed() - 400.0).abs() < 1e-3,
        "speed should be 400, got {}",
        v.speed()
    );
}

#[test]
fn from_angle_up_negative_is_counterclockwise_leftward() {
    let v = Velocity2D::from_angle_up(-std::f32::consts::FRAC_PI_4, 400.0);
    assert!(v.0.x < 0.0, "negative angle should have negative x");
    assert!(v.0.y > 0.0, "PI/4 from up should still have positive y");
}

#[test]
fn from_angle_up_pi_is_straight_down() {
    let v = Velocity2D::from_angle_up(std::f32::consts::PI, 400.0);
    assert!(v.0.x.abs() < 1e-3, "x should be ~0, got {}", v.0.x);
    assert!(
        (v.0.y - (-400.0)).abs() < 1e-3,
        "y should be -400, got {}",
        v.0.y
    );
}

#[test]
fn from_angle_up_half_pi_is_straight_right() {
    let v = Velocity2D::from_angle_up(std::f32::consts::FRAC_PI_2, 400.0);
    assert!(
        (v.0.x - 400.0).abs() < 1e-3,
        "x should be 400, got {}",
        v.0.x
    );
    assert!(v.0.y.abs() < 1e-3, "y should be ~0, got {}", v.0.y);
}

#[test]
fn from_angle_up_neg_half_pi_is_straight_left() {
    let v = Velocity2D::from_angle_up(-std::f32::consts::FRAC_PI_2, 400.0);
    assert!(
        (v.0.x - (-400.0)).abs() < 1e-3,
        "x should be -400, got {}",
        v.0.x
    );
    assert!(v.0.y.abs() < 1e-3, "y should be ~0, got {}", v.0.y);
}

#[test]
fn from_angle_up_three_quarter_pi_is_lower_right() {
    let v = Velocity2D::from_angle_up(3.0 * std::f32::consts::FRAC_PI_4, 400.0);
    assert!(v.0.x > 0.0, "3PI/4 from up should have positive x");
    assert!(v.0.y < 0.0, "3PI/4 from up should have negative y");
    assert!((v.speed() - 400.0).abs() < 1e-3, "speed should be 400");
}

#[test]
fn from_angle_up_neg_three_quarter_pi_is_lower_left() {
    let v = Velocity2D::from_angle_up(-3.0 * std::f32::consts::FRAC_PI_4, 400.0);
    assert!(v.0.x < 0.0, "-3PI/4 from up should have negative x");
    assert!(v.0.y < 0.0, "-3PI/4 from up should have negative y");
    assert!((v.speed() - 400.0).abs() < 1e-3, "speed should be 400");
}

// ── Velocity2D::rotate_by() ────────────────────────────────

#[test]
fn rotate_by_zero_is_unchanged() {
    let v = Velocity2D(Vec2::new(100.0, 300.0));
    let result = v.rotate_by(0.0);
    assert!((result.0.x - v.0.x).abs() < 1e-3, "x should be unchanged");
    assert!((result.0.y - v.0.y).abs() < 1e-3, "y should be unchanged");
}

#[test]
fn rotate_by_preserves_speed() {
    let v = Velocity2D(Vec2::new(3.0, 4.0));
    let result = v.rotate_by(0.5);
    assert!(
        (result.speed() - 5.0).abs() < 1e-3,
        "speed should be preserved at 5.0, got {}",
        result.speed()
    );
}

#[test]
fn rotate_by_positive_rotates_clockwise() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(std::f32::consts::FRAC_PI_2);
    assert!(
        (result.0.x - 400.0).abs() < 1e-3,
        "x should be 400, got {}",
        result.0.x
    );
    assert!(
        result.0.y.abs() < 1e-3,
        "y should be ~0, got {}",
        result.0.y
    );
}

#[test]
fn rotate_by_negative_rotates_counterclockwise() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(-std::f32::consts::FRAC_PI_2);
    assert!(
        (result.0.x - (-400.0)).abs() < 1e-3,
        "x should be -400, got {}",
        result.0.x
    );
    assert!(
        result.0.y.abs() < 1e-3,
        "y should be ~0, got {}",
        result.0.y
    );
}

#[test]
fn rotate_by_small_angle_from_up_tilts_right() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(2.0_f32.to_radians());
    assert!(
        result.0.x > 0.0,
        "small positive rotation should tilt x positive"
    );
    assert!(
        result.0.y > 0.0,
        "small rotation from up should keep y positive"
    );
}

#[test]
fn rotate_by_small_negative_from_up_tilts_left() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(-2.0_f32.to_radians());
    assert!(
        result.0.x < 0.0,
        "small negative rotation should tilt x negative"
    );
    assert!(
        result.0.y > 0.0,
        "small rotation from up should keep y positive"
    );
}

#[test]
fn rotate_by_pi_reverses_direction() {
    let v = Velocity2D(Vec2::new(0.0, 400.0));
    let result = v.rotate_by(std::f32::consts::PI);
    assert!(
        result.0.x.abs() < 1e-3,
        "x should be ~0, got {}",
        result.0.x
    );
    assert!(
        (result.0.y - (-400.0)).abs() < 1e-3,
        "y should be -400, got {}",
        result.0.y
    );
}

#[test]
fn rotate_by_from_diagonal_stays_correct() {
    // Start at 45 degrees (upper-right), rotate 90 clockwise → lower-right
    let v = Velocity2D(Vec2::new(283.0, 283.0)); // ~400 speed at 45 deg
    let result = v.rotate_by(std::f32::consts::FRAC_PI_2);
    assert!(result.0.x > 0.0, "should have positive x (lower-right)");
    assert!(result.0.y < 0.0, "should have negative y (lower-right)");
    assert!(
        (result.speed() - v.speed()).abs() < 1e-1,
        "speed should be preserved"
    );
}

#[test]
fn rotate_by_from_diagonal_counterclockwise() {
    // Start at 45 degrees (upper-right), rotate 90 counterclockwise → upper-left
    let v = Velocity2D(Vec2::new(283.0, 283.0));
    let result = v.rotate_by(-std::f32::consts::FRAC_PI_2);
    assert!(result.0.x < 0.0, "should have negative x (upper-left)");
    assert!(result.0.y > 0.0, "should have positive y (upper-left)");
}

// ── clamp_angle: near-axis symmetry tests ───────────────────

#[test]
fn clamp_angle_near_vertical_right_clamped_away() {
    let v = Velocity2D::from_angle_up(3.0_f32.to_radians(), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.x > 0.0, "should stay right of vertical");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    let upper = std::f32::consts::FRAC_PI_2 - bounds.1;
    assert!(
        (angle - upper).abs() < 1e-3,
        "angle should be clamped to {upper}, got {angle}"
    );
}

#[test]
fn clamp_angle_near_vertical_left_clamped_away() {
    let v = Velocity2D::from_angle_up(-3.0_f32.to_radians(), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.x < 0.0, "should stay left of vertical");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    let upper = std::f32::consts::FRAC_PI_2 - bounds.1;
    assert!(
        (angle - upper).abs() < 1e-3,
        "angle should be clamped to {upper}, got {angle}"
    );
}

#[test]
fn clamp_angle_near_horizontal_above_clamped_away() {
    let v = Velocity2D::from_angle_up(std::f32::consts::FRAC_PI_2 - 3.0_f32.to_radians(), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.y > 0.0, "should stay above horizontal");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (angle - bounds.0).abs() < 1e-3,
        "angle should be clamped to {}, got {angle}",
        bounds.0
    );
}

#[test]
fn clamp_angle_near_horizontal_below_clamped_away() {
    let v = Velocity2D::from_angle_up(std::f32::consts::FRAC_PI_2 + 3.0_f32.to_radians(), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.y < 0.0, "should stay below horizontal");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (angle - bounds.0).abs() < 1e-3,
        "angle should be clamped to {}, got {angle}",
        bounds.0
    );
}

// Near-vertical in negative-y (downward) quadrants

#[test]
fn clamp_angle_near_vertical_down_right_clamped_away() {
    // 3 degrees right of straight down → should clamp to min_from_vertical
    let v = Velocity2D::from_angle_up(std::f32::consts::PI - 3.0_f32.to_radians(), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.x > 0.0, "should stay right of vertical");
    assert!(result.0.y < 0.0, "should stay below horizontal (downward)");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    let upper = std::f32::consts::FRAC_PI_2 - bounds.1;
    assert!(
        (angle - upper).abs() < 1e-3,
        "angle should be clamped to {upper}, got {angle}"
    );
}

#[test]
fn clamp_angle_near_vertical_down_left_clamped_away() {
    // 3 degrees left of straight down → should clamp to min_from_vertical
    let v = Velocity2D::from_angle_up(-(std::f32::consts::PI - 3.0_f32.to_radians()), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.x < 0.0, "should stay left of vertical");
    assert!(result.0.y < 0.0, "should stay below horizontal (downward)");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    let upper = std::f32::consts::FRAC_PI_2 - bounds.1;
    assert!(
        (angle - upper).abs() < 1e-3,
        "angle should be clamped to {upper}, got {angle}"
    );
}

// Near-horizontal in negative-x (leftward) quadrants

#[test]
fn clamp_angle_near_horizontal_left_above_clamped_away() {
    // Moving left, 3 degrees above horizontal
    let v = Velocity2D::from_angle_up(-(std::f32::consts::FRAC_PI_2 - 3.0_f32.to_radians()), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.x < 0.0, "should stay left");
    assert!(result.0.y > 0.0, "should stay above horizontal");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (angle - bounds.0).abs() < 1e-3,
        "angle should be clamped to {}, got {angle}",
        bounds.0
    );
}

#[test]
fn clamp_angle_near_horizontal_left_below_clamped_away() {
    // Moving left, 3 degrees below horizontal
    let v = Velocity2D::from_angle_up(-(std::f32::consts::FRAC_PI_2 + 3.0_f32.to_radians()), 400.0);
    let bounds = (5.0_f32.to_radians(), 5.0_f32.to_radians());
    let result = v.clamp_angle(bounds);
    assert!(result.0.x < 0.0, "should stay left");
    assert!(result.0.y < 0.0, "should stay below horizontal");
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        (angle - bounds.0).abs() < 1e-3,
        "angle should be clamped to {}, got {angle}",
        bounds.0
    );
}

// ── constrained() ───────────────────────────────────────────────────────

#[test]
fn constrained_applies_base_speed() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(&BaseSpeed(400.0), None, None, None, None);
    assert!((result.speed() - 400.0).abs() < 1e-3);
}

#[test]
fn constrained_clamps_to_min_speed() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(
        &BaseSpeed(100.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        None,
        None,
    );
    assert!((result.speed() - 200.0).abs() < 1e-3);
}

#[test]
fn constrained_clamps_to_max_speed() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(
        &BaseSpeed(1000.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        None,
        None,
    );
    assert!((result.speed() - 800.0).abs() < 1e-3);
}

#[test]
fn constrained_no_min_max_uses_permissive_defaults() {
    let v = Velocity2D(Vec2::new(0.0, 1.0));
    let result = v.constrained(&BaseSpeed(500.0), None, None, None, None);
    assert!((result.speed() - 500.0).abs() < 1e-3);
}

#[test]
fn constrained_applies_angle_clamping() {
    // Near-horizontal velocity should be clamped to min angle from horizontal
    let v = Velocity2D(Vec2::new(400.0, 1.0));
    let min_h = MinAngleHorizontal(0.3); // ~17 degrees
    let result = v.constrained(&BaseSpeed(400.0), None, None, Some(&min_h), None);
    let angle = result.0.y.abs().atan2(result.0.x.abs());
    assert!(
        angle >= 0.3 - 1e-3,
        "angle from horizontal should be >= 0.3, got {angle}"
    );
}

#[test]
fn constrained_no_angle_constraints_preserves_direction() {
    let v = Velocity2D(Vec2::new(300.0, 400.0));
    let result = v.constrained(&BaseSpeed(500.0), None, None, None, None);
    let dir_before = v.0.normalize();
    let dir_after = result.0.normalize();
    assert!((dir_before.x - dir_after.x).abs() < 1e-3);
    assert!((dir_before.y - dir_after.y).abs() < 1e-3);
}

#[test]
fn constrained_zero_velocity_returns_zero() {
    let v = Velocity2D(Vec2::ZERO);
    let result = v.constrained(
        &BaseSpeed(400.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        Some(&MinAngleHorizontal(0.1)),
        Some(&MinAngleVertical(0.1)),
    );
    assert_eq!(result.0, Vec2::ZERO);
}

#[test]
fn constrained_all_params_applied() {
    let v = Velocity2D(Vec2::new(0.6, 0.8));
    let result = v.constrained(
        &BaseSpeed(400.0),
        Some(&MinSpeed(200.0)),
        Some(&MaxSpeed(800.0)),
        Some(&MinAngleHorizontal(0.087)),
        Some(&MinAngleVertical(0.087)),
    );
    assert!(
        (result.speed() - 400.0).abs() < 1e-3,
        "speed should be base_speed"
    );
}
