//! Phantom-breaker collision gating — phantom-only world tests.
//!
//! Group B: Tilt is NOT applied on phantom hit.
//! Group C: Spread override is NOT applied on phantom hit.
//! Group D: `LastImpact` is NOT stamped on phantom hit.
//! Group E: Piercing-bolt handling is NOT applied on phantom hit.

use bevy::prelude::*;

use super::helpers::{breaker_y, start_y_above};
use crate::{
    bolt::{
        components::{ImpactSide, LastImpact, PiercingRemaining},
        systems::bolt_breaker_collision::tests::helpers::*,
        test_utils::piercing_stack,
    },
    breaker::components::{BreakerReflectionSpread, BreakerTilt},
    prelude::*,
};

// ── Group B: Tilt is NOT applied on phantom hit ───────────────────────────────
//
// These tests use a PHANTOM-ONLY world (one phantom, no real breaker).
// Today `single()` succeeds and applies tilt unconditionally, so the
// `vel.0.x.abs() < 1e-3` assertions fail (RED).

/// Behavior #4: phantom with non-zero tilt produces a pure vertical flip
/// (no horizontal component from tilt).
#[test]
fn phantom_with_tilt_produces_pure_vertical_reflection() {
    let mut app = test_app();
    let by = breaker_y();

    // Phantom only.
    let phantom_entity = spawn_phantom_breaker_at(&mut app, 0.0, by);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BreakerTilt {
            angle:       0.3,
            ease_start:  0.0,
            ease_target: 0.0,
        });

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x.abs() < 1e-3,
        "phantom tilt should NOT steer bolt horizontally (vel.x={:.4}); today tilt is applied unconditionally",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "bolt should still reflect upward off phantom (vy={:.1})",
        vel.0.y
    );
}

/// Behavior #4 edge case: off-center hit on phantom with tilt — still no
/// horizontal component (both tilt AND spread gated).
#[test]
fn phantom_with_tilt_off_center_hit_produces_pure_vertical_reflection() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();
    let hw = default_breaker_width();

    let phantom_entity = spawn_phantom_breaker_at(&mut app, 0.0, by);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BreakerTilt {
            angle:       0.3,
            ease_start:  0.0,
            ease_target: 0.0,
        });

    // Right-edge hit.
    let hit_x = hw.half_width() - 5.0;
    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x.abs() < 1e-3,
        "phantom off-center hit with tilt should still produce no horizontal steering (vel.x={:.4})",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "bolt should reflect upward (vy={:.1})",
        vel.0.y
    );
}

/// Behavior #4 edge case: negative tilt on phantom — also no horizontal effect.
#[test]
fn phantom_with_negative_tilt_produces_pure_vertical_reflection() {
    let mut app = test_app();
    let by = breaker_y();

    let phantom_entity = spawn_phantom_breaker_at(&mut app, 0.0, by);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BreakerTilt {
            angle:       -0.5,
            ease_start:  0.0,
            ease_target: 0.0,
        });

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x.abs() < 1e-3,
        "negative phantom tilt should have no horizontal effect (vel.x={:.4})",
        vel.0.x
    );
    assert!(vel.0.y > 0.0);
}

// ── Group C: Spread override is NOT applied on phantom hit ───────────────────
//
// Phantom-only world. Today `single()` applies `reflect_top_hit` with the full
// spread formula, so the near-edge assertion `vel.x.abs() < 1e-3` fails (RED).

/// Behavior #5: right-edge hit on phantom with zero tilt produces pure vertical
/// flip — no angle-spread steering.
#[test]
fn phantom_right_edge_hit_produces_pure_vertical_reflection() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();
    let hw = default_breaker_width();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let hit_x = hw.half_width() - 5.0;
    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x.abs() < 1e-3,
        "phantom right-edge hit should produce no horizontal steering from spread (vel.x={:.4}); today spread is applied",
        vel.0.x
    );
    assert!(vel.0.y > 0.0);
}

/// Behavior #5 edge case: large spread value on phantom — still no steering.
#[test]
fn phantom_with_large_spread_still_produces_pure_vertical_reflection() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();
    let hw = default_breaker_width();

    let phantom_entity = spawn_phantom_breaker_at(&mut app, 0.0, by);
    // Override spread to a large value.
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BreakerReflectionSpread(2.0));

    let hit_x = hw.half_width() - 5.0;
    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x.abs() < 1e-3,
        "large spread on phantom should have no effect (vel.x={:.4})",
        vel.0.x
    );
    assert!(vel.0.y > 0.0);
}

/// Behavior #5 edge case: left-edge hit on phantom — also no steering.
#[test]
fn phantom_left_edge_hit_produces_pure_vertical_reflection() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();
    let hw = default_breaker_width();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let hit_x = -hw.half_width() + 5.0;
    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, hit_x, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x.abs() < 1e-3,
        "phantom left-edge hit should produce no horizontal steering (vel.x={:.4})",
        vel.0.x
    );
    assert!(vel.0.y > 0.0);
}

// ── Group D: LastImpact is NOT stamped on phantom hit ────────────────────────
//
// Phantom-only world. Today stamp_last_impact runs unconditionally.

/// Behavior #6: top-surface hit on phantom does not insert `LastImpact`.
#[test]
fn phantom_top_hit_does_not_insert_last_impact() {
    let mut app = test_app();
    let by = breaker_y();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);

    tick(&mut app);

    // Verify the hit happened (bolt reflected upward).
    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(vel.0.y > 0.0, "bolt should have reflected off phantom top");

    let li = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        li.is_none(),
        "phantom top hit should NOT insert LastImpact (got {li:?}); today stamp_last_impact runs unconditionally"
    );
}

/// Behavior #6 edge case: pre-existing `LastImpact` on bolt is NOT overwritten
/// by phantom hit.
#[test]
fn phantom_top_hit_does_not_overwrite_existing_last_impact() {
    let mut app = test_app();
    let by = breaker_y();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut().entity_mut(bolt_entity).insert(LastImpact {
        position: Vec2::new(999.0, 999.0),
        side:     ImpactSide::Left,
    });

    tick(&mut app);

    let li = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("pre-existing LastImpact should remain after phantom hit");
    assert_eq!(
        li.position,
        Vec2::new(999.0, 999.0),
        "phantom hit should NOT overwrite LastImpact.position (got {:?})",
        li.position
    );
    assert_eq!(
        li.side,
        ImpactSide::Left,
        "phantom hit should NOT overwrite LastImpact.side (got {:?})",
        li.side
    );
}

/// Behavior #6 edge case: side-face hit on phantom does not insert `LastImpact`.
#[test]
fn phantom_side_hit_does_not_insert_last_impact() {
    let mut app = test_app();
    let by = breaker_y();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    // Bolt aimed at left face (mirrors `breaker_left_side_rebound_stamps_last_impact_with_left_side`).
    let bolt_entity = spawn_bolt(&mut app, -71.0, by, 400.0, -50.0);

    tick(&mut app);

    let li = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        li.is_none(),
        "phantom side hit should NOT insert LastImpact (got {li:?})"
    );
}

/// Behavior #6 edge case: overlap-resolution hit on phantom does not insert `LastImpact`.
#[test]
fn phantom_overlap_resolution_does_not_insert_last_impact() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let inside_y = by + hh.half_height() + default_bolt_radius().0 - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, inside_y, 0.0, -100.0);

    tick(&mut app);

    let li = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        li.is_none(),
        "phantom overlap-resolution hit should NOT insert LastImpact (got {li:?})"
    );
}

// ── Group E: Piercing-bolt handling is NOT applied on phantom hit ─────────────
//
// Phantom-only world. Today emit_bump resets PiercingRemaining unconditionally.

/// Behavior #7: phantom hit does not reset `PiercingRemaining` from `ActivePiercings`.
#[test]
fn phantom_hit_does_not_reset_piercing_remaining() {
    let mut app = test_app();
    let by = breaker_y();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app);

    // Confirm the hit happened.
    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt should have reflected off phantom (vy={:.1})",
        vel.0.y
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "phantom hit must NOT reset PiercingRemaining to ActivePiercings (expected 0, got {}); today emit_bump resets it to 3",
        pr.0
    );
}

/// Behavior #7 edge case: bolt has `piercing_stack(&[2, 1])` and `PiercingRemaining(1)` —
/// both remain unchanged after phantom hit.
#[test]
fn phantom_hit_does_not_alter_existing_piercing_remaining() {
    let mut app = test_app();
    let by = breaker_y();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[2, 1]), PiercingRemaining(1)));

    tick(&mut app);

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 1,
        "phantom hit must NOT alter PiercingRemaining(1) (got {})",
        pr.0
    );
}

/// Behavior #7 edge case: bolt has no `ActivePiercings` and `PiercingRemaining(5)` —
/// remains 5 after phantom hit.
#[test]
fn phantom_hit_does_not_alter_piercing_remaining_without_active_piercings() {
    let mut app = test_app();
    let by = breaker_y();

    spawn_phantom_breaker_at(&mut app, 0.0, by);

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(PiercingRemaining(5));
    // No ActivePiercings inserted.

    tick(&mut app);

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 5,
        "phantom hit without ActivePiercings must NOT alter PiercingRemaining(5) (got {})",
        pr.0
    );
}
