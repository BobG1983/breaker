//! Phantom-breaker collision gating tests.
//!
//! Phantoms receive only a core velocity flip; tilt, spread, `LastImpact` and
//! `PiercingRemaining` effects are suppressed. In a mixed world the current
//! `single()` call returns `Err(MultipleEntities)` so tests 1-9 fail today.
//! Test 10 is a regression guard that passes today.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{ImpactSide, LastImpact, PiercingRemaining},
        systems::bolt_breaker_collision::tests::helpers::*,
        test_utils::piercing_stack,
    },
    breaker::components::{BreakerReflectionSpread, BreakerTilt},
    prelude::*,
};

// ── Test helpers ──────────────────────────────────────────────────────────────

fn breaker_y() -> f32 {
    -250.0
}

fn start_y_above(breaker_cy: f32) -> f32 {
    let hh = default_breaker_height();
    breaker_cy + hh.half_height() + default_bolt_radius().0 + 3.0
}

// ── Group A: Standard reflection still occurs on phantom hit (mixed world) ───
//
// All three tests use a world with ONE real breaker + ONE phantom breaker.
// Today `breaker_query.single()` returns `Err(MultipleEntities)` in this
// scenario, so the system early-returns and no collision occurs — causing
// each assertion to fail (RED).

/// Behavior #1: bolt aimed at the phantom reflects upward in a mixed world.
/// Edge case sub-test also swaps real/phantom positions to confirm the system
/// iterates all `With<Breaker>` entities regardless of order.
#[test]
fn bolt_aimed_at_phantom_reflects_upward_in_mixed_world() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    // Real breaker on the left; phantom on the right.
    spawn_breaker_at(&mut app, -200.0, by);
    let phantom_entity = spawn_phantom_breaker_at(&mut app, 200.0, by);

    // Bolt aimed at the phantom.
    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 200.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt aimed at phantom should reflect upward (vy={:.1}); today single() returns Err(MultipleEntities)",
        vel.0.y
    );
    let _ = phantom_entity; // referenced above in spawn
}

/// Behavior #1 edge case: real and phantom swapped — bolt still reflects off
/// whichever entity it is aimed at.
#[test]
fn bolt_aimed_at_phantom_reflects_upward_in_mixed_world_swapped_positions() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    // Phantom on the left; real breaker on the right.
    let _phantom_entity = spawn_phantom_breaker_at(&mut app, -200.0, by);
    spawn_breaker_at(&mut app, 200.0, by);

    // Bolt aimed at the phantom (left side).
    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, -200.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt aimed at phantom (left slot) should reflect upward (vy={:.1})",
        vel.0.y
    );
}

/// Behavior #2: `BoltImpactBreaker` is emitted with phantom entity as `breaker`
/// when bolt hits the phantom in a mixed world.
#[test]
fn bolt_impact_breaker_message_carries_phantom_entity_in_mixed_world() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    app.insert_resource(CapturedHitPairs::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_pairs.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    // Real on the left; phantom on the right.
    spawn_breaker_at(&mut app, -200.0, by);
    let phantom_entity = spawn_phantom_breaker_at(&mut app, 200.0, by);

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 200.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let captured = app.world().resource::<CapturedHitPairs>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one BoltImpactBreaker should be emitted (got {}); today single() returns Err so 0 messages",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].0, bolt_entity,
        "BoltImpactBreaker.bolt should be the bolt entity"
    );
    assert_eq!(
        captured.0[0].1, phantom_entity,
        "BoltImpactBreaker.breaker should be the phantom entity, not the real one"
    );
}

/// Behavior #2 edge case: bolt aimed at the REAL breaker in a mixed world.
/// Exactly one message with `breaker == real_entity`.
#[test]
fn bolt_impact_breaker_message_carries_real_entity_when_aimed_at_real_in_mixed_world() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    app.insert_resource(CapturedHitPairs::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_pairs.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    // Real on the left; phantom on the right.
    let real_entity = spawn_breaker_at(&mut app, -200.0, by);
    spawn_phantom_breaker_at(&mut app, 200.0, by);

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, -200.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let captured = app.world().resource::<CapturedHitPairs>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one BoltImpactBreaker should be emitted (got {})",
        captured.0.len()
    );
    assert_eq!(captured.0[0].0, bolt_entity);
    assert_eq!(
        captured.0[0].1, real_entity,
        "BoltImpactBreaker.breaker should be the real entity when bolt aimed at real"
    );
}

/// Behavior #3: bolt inside phantom's AABB is repositioned above the phantom
/// surface and reflected upward in a mixed world.
#[test]
fn bolt_inside_phantom_aabb_is_repositioned_above_and_reflected_in_mixed_world() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    // Real on the left; phantom on the right.
    spawn_breaker_at(&mut app, -200.0, by);
    spawn_phantom_breaker_at(&mut app, 200.0, by);

    // Bolt inside phantom's expanded AABB (overlap scenario).
    let inside_y = by + hh.half_height() + default_bolt_radius().0 - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 200.0, inside_y, 0.0, -100.0);

    tick(&mut app);

    let pos = app.world().get::<Position2D>(bolt_entity).unwrap();
    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    let above_y = by + hh.half_height() + default_bolt_radius().0;

    assert!(
        pos.0.y >= above_y - 0.001,
        "bolt should be repositioned above phantom surface (>= {above_y:.3}), got {:.3}",
        pos.0.y
    );
    assert!(
        vel.0.y > 0.0,
        "bolt should reflect upward after overlap resolution on phantom (vy={:.1})",
        vel.0.y
    );
}

/// Behavior #3 edge case: upward-moving bolt inside phantom AABB is repositioned
/// but NOT reflected (upward bolt skips the reflection branch).
#[test]
fn upward_bolt_inside_phantom_aabb_is_repositioned_but_not_reflected_in_mixed_world() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    spawn_breaker_at(&mut app, -200.0, by);
    spawn_phantom_breaker_at(&mut app, 200.0, by);

    let inside_y = by + hh.half_height() + default_bolt_radius().0 - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 200.0, inside_y, 0.0, 100.0);

    tick(&mut app);

    let pos = app.world().get::<Position2D>(bolt_entity).unwrap();
    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    let above_y = by + hh.half_height() + default_bolt_radius().0;

    assert!(
        pos.0.y >= above_y - 0.001,
        "upward bolt inside phantom AABB should still be pushed above surface (>= {above_y:.3}), got {:.3}",
        pos.0.y
    );
    assert!(
        vel.0.y > 0.0,
        "upward bolt should remain upward after overlap push (no reflection); vy={:.1}",
        vel.0.y
    );
}

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

// ── Group F: Real-breaker effects still fire in a mixed world ─────────────────
//
// Mixed world. Today single() returns Err → system no-ops → all assertions fail.

/// Behavior #8: real breaker in a mixed world still gets full effects
/// (tilt steering, `LastImpact`, piercing reset, message) when bolt aimed at real.
#[test]
fn real_breaker_gets_full_effects_in_mixed_world() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    app.insert_resource(CapturedHitPairs::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_pairs.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    // Real breaker on the left with tilt; phantom on the right.
    let real_entity = spawn_breaker_at(&mut app, -200.0, by);
    app.world_mut().entity_mut(real_entity).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });
    spawn_phantom_breaker_at(&mut app, 200.0, by);

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, -200.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x > 0.0,
        "real breaker tilt (0.3) should steer bolt rightward (vel.x={:.4}); today single() returns Err",
        vel.0.x
    );

    let li = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("real breaker hit should stamp LastImpact");
    assert_eq!(
        li.side,
        ImpactSide::Top,
        "real breaker top hit should stamp ImpactSide::Top (got {:?})",
        li.side
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 3,
        "real breaker hit should reset PiercingRemaining to 3 (got {})",
        pr.0
    );

    let captured = app.world().resource::<CapturedHitPairs>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one BoltImpactBreaker message (got {})",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].1, real_entity,
        "message breaker field should be real_entity"
    );
}

/// Behavior #8 edge case: phantom's large spread does not bleed into real-breaker
/// reflection when bolt aimed at real.
#[test]
fn phantom_spread_does_not_bleed_into_real_breaker_reflection() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    // Real with default spread; phantom with large spread.
    let real_entity = spawn_breaker_at(&mut app, -200.0, by);
    let phantom_entity = spawn_phantom_breaker_at(&mut app, 200.0, by);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BreakerReflectionSpread(2.0));

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, -200.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "bolt aimed at real should reflect upward (vy={:.1})",
        vel.0.y
    );
    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 3,
        "real breaker should still reset PiercingRemaining to 3 (got {})",
        pr.0
    );

    let _ = real_entity;
}

// ── Group G: Phantom gating still holds in a mixed world ─────────────────────
//
// Mixed world. Today single() no-ops everything → all assertions fail.

/// Behavior #9: phantom in a mixed world receives only standard reflection
/// (no tilt, no `LastImpact`, no piercing reset) when bolt aimed at phantom.
#[test]
fn phantom_in_mixed_world_receives_only_standard_reflection() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    app.insert_resource(CapturedHitPairs::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_pairs.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    // Both breakers carry the same non-zero tilt to prove the gate is per-entity.
    let real_entity = spawn_breaker_at(&mut app, -200.0, by);
    app.world_mut().entity_mut(real_entity).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });
    let phantom_entity = spawn_phantom_breaker_at(&mut app, 200.0, by);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(BreakerTilt {
            angle:       0.3,
            ease_start:  0.0,
            ease_target: 0.0,
        });

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 200.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x.abs() < 1e-3,
        "phantom tilt should NOT steer bolt even with identical tilt on real (vel.x={:.4}); today single() returns Err",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "bolt should still reflect upward off phantom (vy={:.1})",
        vel.0.y
    );

    let li = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        li.is_none(),
        "phantom in mixed world should NOT stamp LastImpact (got {li:?})"
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 0,
        "phantom in mixed world must NOT reset PiercingRemaining (got {})",
        pr.0
    );

    let captured = app.world().resource::<CapturedHitPairs>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one BoltImpactBreaker message (got {})",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].1, phantom_entity,
        "message breaker field should be phantom_entity"
    );
}

/// Behavior #9 edge case: phantom with no `ActivePiercings` and no `PiercingRemaining`
/// — phantom hit does not INSERT `PiercingRemaining`.
#[test]
fn phantom_in_mixed_world_does_not_insert_piercing_remaining() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    spawn_breaker_at(&mut app, -200.0, by);
    let phantom_entity = spawn_phantom_breaker_at(&mut app, 200.0, by);

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 200.0, start_y, 0.0, -400.0);
    // No ActivePiercings, no PiercingRemaining on the bolt.

    tick(&mut app);

    let pr = app.world().get::<PiercingRemaining>(bolt_entity);
    assert!(
        pr.is_none(),
        "phantom hit must NOT insert PiercingRemaining on a bolt that has none (got {pr:?})"
    );

    let _ = phantom_entity;
}

// ── Group H: Regression guard — single real breaker baseline ─────────────────
//
// This test PASSES today. It is the "do not break the real breaker" guard.
// If any of the above gating work accidentally breaks single-real-breaker
// behavior, this test will catch it.

/// Behavior #10 (regression guard): single real breaker (no phantom) still
/// applies all effects — reflects upward, stamps `LastImpact::Top`, resets
/// `PiercingRemaining`, emits `BoltImpactBreaker`.
#[test]
fn single_real_breaker_baseline_is_unchanged() {
    let mut app = test_app();
    let by = breaker_y();
    let hh = default_breaker_height();

    app.insert_resource(CapturedHitPairs::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_pairs.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    let real_entity = spawn_breaker_at(&mut app, 0.0, by);

    let start_y = by + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[3]), PiercingRemaining(0)));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.y > 0.0,
        "single real breaker: bolt should reflect upward (vy={:.1})",
        vel.0.y
    );

    let li = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("single real breaker: LastImpact should be stamped");
    assert_eq!(
        li.side,
        ImpactSide::Top,
        "single real breaker: ImpactSide::Top expected (got {:?})",
        li.side
    );

    let pr = app.world().get::<PiercingRemaining>(bolt_entity).unwrap();
    assert_eq!(
        pr.0, 3,
        "single real breaker: PiercingRemaining should be reset to 3 (got {})",
        pr.0
    );

    let captured = app.world().resource::<CapturedHitPairs>();
    assert_eq!(
        captured.0.len(),
        1,
        "single real breaker: exactly one BoltImpactBreaker message (got {})",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].1, real_entity,
        "BoltImpactBreaker.breaker should be real_entity"
    );
}

/// Behavior #10 edge case: real breaker with tilt still steers when no phantom present.
#[test]
fn single_real_breaker_with_tilt_still_steers() {
    let mut app = test_app();
    let by = breaker_y();

    let real_entity = spawn_breaker_at(&mut app, 0.0, by);
    app.world_mut().entity_mut(real_entity).insert(BreakerTilt {
        angle:       0.3,
        ease_start:  0.0,
        ease_target: 0.0,
    });

    let start_y = start_y_above(by);
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt_entity).unwrap();
    assert!(
        vel.0.x > 0.0,
        "single real breaker tilt should steer bolt rightward (vel.x={:.4})",
        vel.0.x
    );
}
