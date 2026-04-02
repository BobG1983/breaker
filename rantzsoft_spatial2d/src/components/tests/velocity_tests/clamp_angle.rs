use bevy::prelude::*;

use super::super::super::definitions::*;

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
