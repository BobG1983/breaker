use super::super::super::definitions::*;

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
