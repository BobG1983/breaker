//! Behavior 11: Shield reflects velocity and clamps position.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{super::helpers::*, helpers::spawn_shielded_breaker};
use crate::{bolt::components::Bolt, shared::PlayfieldConfig};

// ── Behavior 11: Shield reflects velocity and clamps position ──

#[test]
fn shield_reflects_velocity_and_clamps_position() {
    // Given: Bolt at (50.0, -316.0) with velocity (360.0, -623.5) (magnitude ~720.0,
    //        matching BaseSpeed from the definition so apply_velocity_formula is a no-op).
    //        Definition bolt has radius 14.0.
    // When: bolt_lost runs
    // Then: Velocity becomes (360.0, 623.5) — Y abs(), X preserved.
    //       Position Y clamped to bottom() + radius = -300.0 + 14.0 = -286.0.
    //       Position X preserved at 50.0.
    let mut app = test_app();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5);

    let original_vel = Vec2::new(360.0, -623.5);
    let original_magnitude = original_vel.length();

    spawn_bolt(&mut app, Vec2::new(50.0, -316.0), original_vel);
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.x - 360.0).abs() < 1.0,
        "shield reflect should preserve X component (360.0), got {:.1}",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "shield reflect should make Y positive, got {:.1}",
        vel.0.y
    );
    let new_magnitude = vel.0.length();
    assert!(
        (new_magnitude - original_magnitude).abs() < 1.0,
        "shield reflect should preserve magnitude ({original_magnitude:.1}), got {new_magnitude:.1}"
    );

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    let playfield = PlayfieldConfig::default();
    // Definition bolt has radius 14.0
    let expected_y = playfield.bottom() + 14.0;
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "shield-saved bolt X should be preserved at 50.0, got {:.1}",
        pos.0.x
    );
    assert!(
        (pos.0.y - expected_y).abs() < f32::EPSILON,
        "shield-saved bolt Y should be clamped to bottom() + radius ({expected_y:.1}), got {:.1}",
        pos.0.y
    );
}

#[test]
fn shield_reflects_zero_velocity_unchanged() {
    // Edge case: Velocity (0.0, 0.0) — remains (0.0, 0.0) after reflection.
    // This is a degenerate case. The bolt is still below floor so it will be
    // detected as lost, and the shield reflect produces (0.0, 0.0.abs()) = (0.0, 0.0).
    // The test just verifies no panic and the velocity is "reflected" (trivially).
    let mut app = test_app();
    spawn_shielded_breaker(&mut app, Vec2::new(0.0, -250.0), 5);

    spawn_bolt(&mut app, Vec2::new(0.0, -315.0), Vec2::new(0.0, 0.0));
    tick(&mut app);

    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        (vel.0.x).abs() < f32::EPSILON,
        "zero velocity reflect vx should be 0.0, got {:.3}",
        vel.0.x
    );
    assert!(
        (vel.0.y).abs() < f32::EPSILON,
        "zero velocity reflect vy should be 0.0, got {:.3}",
        vel.0.y
    );
}
