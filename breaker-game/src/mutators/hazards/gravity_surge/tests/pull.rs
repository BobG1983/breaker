//! Group D — `gravity_well_pull` emits `ApplyBoltForce` (migration from direct
//!            `Velocity2D` write).
//!
//! Every test wires only `gravity_well_pull` via `wire_pull_only`.
//! After the migration, `gravity_well_pull` emits `ApplyBoltForce` messages
//! rather than mutating `Velocity2D` directly. Tests assert on message
//! emission; `Velocity2D` remains unchanged because the consumer
//! (`apply_bolt_forces`) is NOT wired in these tests.
//!
//! Integration tests (Group I) and wire-ordering tests (Group J) live in
//! `pull_integration.rs` (split out per the 800-line file-splitting rule).

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{
    super::system::GravityWell,
    helpers::{spawn_bolt, spawn_well, test_app_playing, tick_with_dt, wire_pull_only},
};
use crate::{bolt::messages::ApplyBoltForce, shared::test_utils::collector::MessageCollector};

// ── Group D — gravity_well_pull emits ApplyBoltForce ─────────────────────────

// ── Behavior 1 — single well emits ApplyBoltForce in -X direction ────────────

#[test]
fn single_well_emits_apply_bolt_force_in_negative_x_direction() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let _well = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one ApplyBoltForce message, got {}",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].bolt, bolt,
        "message.bolt must match the spawned bolt entity"
    );
    assert!(
        collector.0[0].force.x < 0.0,
        "expected negative X force (well is to the -X side), got {}",
        collector.0[0].force.x
    );
    assert!(
        collector.0[0].force.y.abs() < 1e-5,
        "expected zero Y force, got {}",
        collector.0[0].force.y
    );
}

#[test]
fn single_well_emits_force_velocity_unchanged_without_consumer() {
    // Edge case: bolt's Velocity2D MUST remain at Vec2::ZERO because the
    // producer no longer writes velocity directly — without apply_bolt_forces
    // wired, velocity is unchanged this tick.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    // One message emitted.
    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);

    // But Velocity2D must be UNCHANGED — producer does not write velocity.
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(
        vel.0,
        Vec2::ZERO,
        "Velocity2D must be unchanged when consumer is not wired, got {:?}",
        vel.0
    );
}

// ── Behavior 2 — single well emits exact acceleration vector ─────────────────

#[test]
fn single_well_emits_exact_acceleration_vector() {
    // accel = strength / distance = 500 / 100 = 5 toward -X.
    // force = Vec2::new(-5.0, 0.0). Independent of dt.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let _bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    assert!(
        (collector.0[0].force - Vec2::new(-5.0, 0.0)).length() < 1e-3,
        "expected force ≈ (-5.0, 0.0), got {:?}",
        collector.0[0].force
    );
}

#[test]
fn single_well_emits_force_independent_of_dt() {
    // Edge case: re-running with dt = 0.1 MUST still yield force ≈ (-5.0, 0.0).
    // The producer emits acceleration, not acceleration*dt.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].bolt, bolt,
        "message.bolt must match the spawned bolt entity"
    );
    assert!(
        (collector.0[0].force - Vec2::new(-5.0, 0.0)).length() < 1e-3,
        "force must be acceleration (dt-independent): expected ≈ (-5.0, 0.0), got {:?}",
        collector.0[0].force
    );
}

// ── Behavior 3 — pre-existing bolt velocity is NOT in the emitted force ───────

#[test]
fn pre_existing_bolt_velocity_absent_from_emitted_force() {
    // The bolt's existing velocity does NOT appear in the emitted force field.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let _bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    // force is pure acceleration, not velocity + acceleration.
    assert!(
        (collector.0[0].force - Vec2::new(-5.0, 0.0)).length() < 1e-3,
        "force must be pure acceleration (no bolt velocity mixed in): expected ≈ (-5.0, 0.0), got {:?}",
        collector.0[0].force
    );
}

#[test]
fn bolt_velocity_unchanged_when_existing_velocity_present_and_consumer_not_wired() {
    // Edge case: bolt's Velocity2D is still (3.0, 4.0) — consumer not wired.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(
        vel.0,
        Vec2::new(3.0, 4.0),
        "Velocity2D must be untouched (consumer not wired): got {:?}",
        vel.0
    );
}

// ── Behavior 4 — bolt at distance < MIN_PULL_DISTANCE clamps the magnitude ───

#[test]
fn bolt_inside_min_pull_distance_emits_clamped_force() {
    // MIN_PULL_DISTANCE = 20.0. Bolt at distance 1.
    //   delta = (-1, 0), distance_clamped = 20.0
    //   direction = (-1/20, 0) = (-0.05, 0)
    //   accel = direction * (500/20) = (-0.05, 0) * 25 = (-1.25, 0)
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(1.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one ApplyBoltForce message"
    );
    assert_eq!(collector.0[0].bolt, bolt);
    // Clamped magnitude must be well below unclamped (~500).
    assert!(
        collector.0[0].force.length() < 100.0,
        "force should be clamped (<100), got {}",
        collector.0[0].force.length()
    );
    assert!(
        (collector.0[0].force.x - (-1.25)).abs() < 1e-3,
        "expected force.x ≈ -1.25, got {}",
        collector.0[0].force.x
    );
    assert!(
        collector.0[0].force.y.abs() < 1e-5,
        "expected force.y ≈ 0.0, got {}",
        collector.0[0].force.y
    );
}

// ── Behavior 5 — clamp keeps direction NOT re-normalised ─────────────────────

#[test]
fn clamped_off_axis_force_keeps_direction_uncorrected() {
    // Bolt at (0.5, 0.866) — direction NOT re-normalised after clamp.
    //   delta = (-0.5, -0.866), delta.length() ≈ 1.0 < 20 → clamp = 20
    //   direction = (-0.5/20, -0.866/20) = (-0.025, -0.0433)
    //   accel = direction * 25 = (-0.625, -1.0825)
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    spawn_bolt(&mut app, Vec2::new(0.5, 0.866), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    let force = collector.0[0].force;
    assert!(
        (force.length() - 1.25).abs() < 1e-3,
        "expected force magnitude ≈ 1.25, got {}",
        force.length()
    );
    assert!(
        (force.x - (-0.625)).abs() < 1e-2,
        "expected force.x ≈ -0.625, got {}",
        force.x
    );
    assert!(
        (force.y - (-1.0825)).abs() < 1e-2,
        "expected force.y ≈ -1.0825, got {}",
        force.y
    );
}

// ── Behavior 6 — bolt at exactly MIN_PULL_DISTANCE → force = -25 in X ────────

#[test]
fn bolt_at_exact_min_pull_distance_emits_twenty_five() {
    // distance = 20.0 → distance.max(20.0) = 20, direction = (-1, 0),
    // accel = -1 * (500/20) = -25.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    spawn_bolt(&mut app, Vec2::new(20.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    let force = collector.0[0].force;
    assert!(
        (force.x - (-25.0)).abs() < 1e-3,
        "expected force.x ≈ -25.0, got {}",
        force.x
    );
    assert!(
        force.y.abs() < 1e-5,
        "expected force.y ≈ 0.0, got {}",
        force.y
    );
}

// ── Behavior 7 — bolt just above MIN_PULL_DISTANCE uses actual distance ───────

#[test]
fn bolt_just_above_min_pull_distance_uses_actual_distance() {
    // Bolt at (20.0001, 0.0) — distance just above clamp; formula uses
    // actual distance → accel ≈ 500/20.0001 ≈ 24.9999.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    spawn_bolt(&mut app, Vec2::new(20.0001, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    assert!(
        (collector.0[0].force.x - (-24.9999)).abs() < 1e-2,
        "expected force.x ≈ -24.9999, got {}",
        collector.0[0].force.x
    );
}

// ── Behavior 8 — bolt on top of well produces zero finite force ───────────────

#[test]
fn bolt_on_top_of_well_emits_zero_finite_force() {
    // delta = 0, distance_clamped = 20, direction = 0/20 = 0, accel = 0.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    let force = collector.0[0].force;
    assert!(
        force.x.is_finite() && force.y.is_finite(),
        "force components must be finite, got {force:?}",
    );
    assert!(
        force.length() < 1e-4,
        "force magnitude must be near zero, got {}",
        force.length()
    );
}

#[test]
fn bolt_essentially_at_well_emits_tiny_finite_force() {
    // Edge case: bolt at (1e-9, 0) — tiny finite delta, tiny finite force.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    spawn_bolt(&mut app, Vec2::new(1e-9, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    let force = collector.0[0].force;
    assert!(
        force.x.is_finite() && force.y.is_finite(),
        "force components must be finite, got {force:?}",
    );
    assert!(
        force.length() < 1e-6,
        "force magnitude must be tiny, got {}",
        force.length()
    );
}

// ── Behavior 9 — producer pre-sums all wells into exactly ONE message per bolt ─

#[test]
fn three_wells_produce_exactly_one_summed_message_per_bolt() {
    // Three wells at (-100,0), (+100,0), (0,+100). Bolt at origin.
    // Well (-100,0): delta=(-100,0), dist=100, dir=(-1,0), contrib=(-5,0)
    // Well (+100,0): delta=(+100,0), dist=100, dir=(+1,0), contrib=(+5,0)
    // Well (0,+100): delta=(0,+100), dist=100, dir=(0,+1), contrib=(0,+5)
    // Vector sum: (0, +5)
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, 100.0), 500.0, 2.0);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    // ONE message, not one per well.
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly 1 ApplyBoltForce message (pre-summed), got {} (regression: producer emitted one message per well)",
        collector.0.len()
    );
    assert_eq!(collector.0[0].bolt, bolt_a);
    assert!(
        collector.0[0].force.x.abs() < 1e-4,
        "X contributions cancel: expected force.x ≈ 0, got {}",
        collector.0[0].force.x
    );
    assert!(
        (collector.0[0].force.y - 5.0).abs() < 1e-3,
        "expected force.y ≈ +5.0, got {}",
        collector.0[0].force.y
    );
}

#[test]
fn three_wells_one_summed_message_edge_case_third_below() {
    // Edge case A: third well at (0, -100) → only message has force ≈ (0, -5).
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, -100.0), 500.0, 2.0);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].bolt, bolt_a);
    assert!(
        collector.0[0].force.x.abs() < 1e-4,
        "expected force.x ≈ 0, got {}",
        collector.0[0].force.x
    );
    assert!(
        (collector.0[0].force.y - (-5.0)).abs() < 1e-3,
        "expected force.y ≈ -5.0, got {}",
        collector.0[0].force.y
    );
}

#[test]
fn three_wells_one_summed_message_edge_case_closer_below_well() {
    // Edge case B: third well at (0, -50) (closer) → force.y ≈ -10.
    // 500/50 = 10, direction (0,-1), contribution (0,-10).
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, -50.0), 500.0, 2.0);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].bolt, bolt_a);
    assert!(
        collector.0[0].force.x.abs() < 1e-4,
        "expected force.x ≈ 0, got {}",
        collector.0[0].force.x
    );
    assert!(
        (collector.0[0].force.y - (-10.0)).abs() < 1e-3,
        "expected force.y ≈ -10.0, got {}",
        collector.0[0].force.y
    );
}

// ── Behavior 10 — well with remaining <= 0 contributes NO force ───────────────

#[test]
fn expired_wells_emit_no_apply_bolt_force() {
    // Both wells expire this tick (0.05 → -0.05; 0.0 → -0.1). Neither passes
    // the positive-remaining filter. No messages emitted.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 0.05);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 0.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert!(
        collector.0.is_empty(),
        "expected zero messages (both wells expired), got {}",
        collector.0.len()
    );
    // Velocity must also be unchanged (no consumer wired, no messages).
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(
        vel.0,
        Vec2::ZERO,
        "bolt velocity must be unchanged when no messages emitted: got {:?}",
        vel.0
    );
}

#[test]
fn expired_wells_filtered_active_well_emits_one_message() {
    // Edge case: two expiring + one surviving well. Exactly one message with
    // the surviving well's contribution.
    // Surviving well at (0,-100): delta=(0,-100), dir=(0,-1), accel=(0,-5).
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 0.05);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 0.0);
    spawn_well(&mut app, Vec2::new(0.0, -100.0), 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one message from the surviving well"
    );
    assert_eq!(collector.0[0].bolt, bolt);
    assert!(
        collector.0[0].force.x.abs() < 1e-4,
        "expected force.x ≈ 0, got {}",
        collector.0[0].force.x
    );
    // Producer emits acceleration (-5 in Y); dt-independent.
    assert!(
        (collector.0[0].force.y - (-5.0)).abs() < 1e-3,
        "expected force.y ≈ -5.0 (acceleration, not velocity), got {}",
        collector.0[0].force.y
    );
}

// ── Behavior 11 — well.remaining ticks down by dt ────────────────────────────
// (Preserved unchanged — same behavior, no message check needed.)

#[test]
fn well_remaining_ticks_down() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

    let remaining = app.world().get::<GravityWell>(well).unwrap().remaining;
    assert!((remaining - 1.5).abs() < 1e-5);
}

#[test]
fn well_remaining_ticks_down_accumulates_across_ticks() {
    // Edge: 0.5s then 0.3s → remaining = 2.0 - 0.5 - 0.3 = 1.2.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.3));

    let remaining = app.world().get::<GravityWell>(well).unwrap().remaining;
    assert!((remaining - 1.2).abs() < 1e-5);
}

// ── Behavior 12 — two bolts each receive their own message ───────────────────

#[test]
fn two_bolts_each_receive_own_apply_bolt_force_message() {
    // bolt_a at (100, 0): force ≈ (-5, 0) — distance 100, accel 5 in -X.
    // bolt_b at (0, 50): force ≈ (0, -10) — distance 50, accel 10 in -Y.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt_a = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::new(0.0, 50.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected exactly 2 ApplyBoltForce messages (one per bolt), got {}",
        collector.0.len()
    );

    // Find the message for each bolt by the `bolt` field.
    let msg_a = collector
        .0
        .iter()
        .find(|m| m.bolt == bolt_a)
        .expect("expected an ApplyBoltForce message for bolt_a");
    let msg_b = collector
        .0
        .iter()
        .find(|m| m.bolt == bolt_b)
        .expect("expected an ApplyBoltForce message for bolt_b");

    // bolt_a: distance 100, force 5 in -X.
    assert!(
        (msg_a.force - Vec2::new(-5.0, 0.0)).length() < 1e-3,
        "bolt_a: expected force ≈ (-5.0, 0.0), got {:?}",
        msg_a.force
    );
    // bolt_b: distance 50, force 10 in -Y.
    assert!(
        (msg_b.force - Vec2::new(0.0, -10.0)).length() < 1e-3,
        "bolt_b: expected force ≈ (0.0, -10.0), got {:?}",
        msg_b.force
    );
}

#[test]
fn third_bolt_gets_correct_directional_force_message() {
    // Edge: bolt_c at (-50, -50), distance = 50*sqrt(2) ≈ 70.71.
    // direction from bolt to well = (+1, +1)/sqrt(2), accel per-axis ≈ 5.0.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt_c = spawn_bolt(&mut app, Vec2::new(-50.0, -50.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].bolt, bolt_c);
    assert!(
        (collector.0[0].force.x - 5.0).abs() < 1e-2,
        "expected force.x ≈ +5.0 (toward well), got {}",
        collector.0[0].force.x
    );
    assert!(
        (collector.0[0].force.y - 5.0).abs() < 1e-2,
        "expected force.y ≈ +5.0 (toward well), got {}",
        collector.0[0].force.y
    );
}

// ── Behavior 13 — zero wells → no message emitted ────────────────────────────

#[test]
fn zero_wells_in_world_no_apply_bolt_force_emitted() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert!(
        collector.0.is_empty(),
        "expected zero messages with no wells, got {}",
        collector.0.len()
    );
    // Velocity unchanged — no consumer wired, no messages.
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(3.0, 4.0));
}

#[test]
fn zero_wells_second_tick_still_no_message() {
    // Edge: second tick — still zero messages, velocity unchanged.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert!(
        collector.0.is_empty(),
        "expected zero messages on second tick with no wells, got {}",
        collector.0.len()
    );
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(3.0, 4.0));
}

// ── Behavior 14 — zero bolts → no panic; wells still tick ────────────────────

#[test]
fn zero_bolts_in_world_no_panic_wells_still_tick_no_messages() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

    // No panic — wells still tick.
    let remaining = app.world().get::<GravityWell>(well).unwrap().remaining;
    assert!((remaining - 1.5).abs() < 1e-5);
    // Zero messages emitted (no bolts).
    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert!(
        collector.0.is_empty(),
        "expected zero messages with no bolts, got {}",
        collector.0.len()
    );
}

#[test]
fn non_bolt_entity_excluded_from_apply_bolt_force() {
    // Edge: non-Bolt entity with Position2D + Velocity2D is excluded by the
    // With<Bolt> filter — no message emitted for it, its velocity is unchanged.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let non_bolt = app
        .world_mut()
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(5.0, 5.0))))
        .id();

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    // No messages (non-Bolt filtered out).
    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert!(
        collector.0.is_empty(),
        "expected zero messages (no Bolt entities), got {}",
        collector.0.len()
    );
    // Non-bolt velocity unchanged.
    let vel = app.world().get::<Velocity2D>(non_bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(5.0, 5.0));
}

// ── Behavior 15 — multi-well multi-bolt with expired-well filter ──────────────

#[test]
fn multi_well_multi_bolt_with_expired_wells_emits_two_messages() {
    // Four wells: two active at (0,+100) and (0,-100) (strength 200,
    // remaining 2.0); two expiring at (+50,0) (remaining 0.0) and
    // (-50,0) (remaining 0.05).
    // With dt=0.1: the expiring wells decrement to negative — excluded.
    // Two bolts.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(0.0, 100.0), 200.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, -100.0), 200.0, 2.0);
    spawn_well(&mut app, Vec2::new(50.0, 0.0), 200.0, 0.0);
    spawn_well(&mut app, Vec2::new(-50.0, 0.0), 200.0, 0.05);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::new(10.0, 10.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let collector = app.world().resource::<MessageCollector<ApplyBoltForce>>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected exactly 2 ApplyBoltForce messages (one per bolt), got {}",
        collector.0.len()
    );

    // bolt_a at origin: symmetric (0,+100) and (0,-100) wells cancel on Y;
    // no X contribution from surviving wells. Assert per-axis to catch
    // asymmetric regressions.
    let msg_a = collector
        .0
        .iter()
        .find(|m| m.bolt == bolt_a)
        .expect("expected an ApplyBoltForce message for bolt_a");
    assert!(
        msg_a.force.x.abs() < 1e-4,
        "bolt_a: expected force.x ≈ 0 (no X contribution from ±Y wells), got {}",
        msg_a.force.x
    );
    assert!(
        msg_a.force.y.abs() < 1e-4,
        "bolt_a: expected force.y ≈ 0 (symmetric ±Y wells cancel), got {}",
        msg_a.force.y
    );

    // bolt_b at (10, 10): modest non-zero force from two surviving wells.
    let msg_b = collector
        .0
        .iter()
        .find(|m| m.bolt == bolt_b)
        .expect("expected an ApplyBoltForce message for bolt_b");
    assert!(
        msg_b.force.length() > 0.0,
        "bolt_b: expected non-zero force, got {:?}",
        msg_b.force
    );
    assert!(
        msg_b.force.length() < 100.0,
        "bolt_b: expected modest force (<100), got {}",
        msg_b.force.length()
    );
}

#[test]
fn multi_well_multi_bolt_preserves_all_four_entities_this_frame() {
    // Edge: despawn is NOT wired — all four wells still exist after the tick.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(0.0, 100.0), 200.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, -100.0), 200.0, 2.0);
    spawn_well(&mut app, Vec2::new(50.0, 0.0), 200.0, 0.0);
    spawn_well(&mut app, Vec2::new(-50.0, 0.0), 200.0, 0.05);
    spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 4);
}
