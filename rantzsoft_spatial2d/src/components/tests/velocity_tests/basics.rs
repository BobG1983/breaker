use bevy::prelude::*;

use crate::components::definitions::*;

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
