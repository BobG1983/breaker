//! Group D — `gravity_well_pull` system.
//!
//! Every test wires only `gravity_well_pull` via `wire_pull_only`. Wells
//! are seeded via `spawn_well`; bolts via `spawn_bolt`.
//! `MIN_PULL_DISTANCE = 20.0` is the file-private distance floor — tests
//! that exercise it use the literal `20.0`.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::{
    super::system::GravityWell,
    helpers::{spawn_bolt, spawn_well, test_app_playing, tick_with_dt, wire_pull_only},
};

// ── Behavior 24 — single well pulls bolt toward it — PRESERVED ──────────

#[test]
fn single_well_pulls_bolt_toward_it() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let _well = spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Well is to the left → accel in -X. Distance = 100, force = 500/100 = 5 u/s²
    // After 1s, velocity = 5 u/s in -X.
    assert!(
        vel.0.x < 0.0,
        "expected negative X velocity, got {:?}",
        vel.0
    );
    assert!(vel.0.y.abs() < 1e-5);
}

#[test]
fn single_well_pull_pins_exact_velocity() {
    // Expanded: pin exact (-5, 0) velocity.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!((vel.0.x - (-5.0)).abs() < 1e-3);
    assert!(vel.0.y.abs() < 1e-5);
}

#[test]
fn pull_impulse_adds_to_existing_velocity() {
    // Edge: pull impulse is added, not overwriting, the existing velocity.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    let expected = Vec2::new(-2.0, 4.0);
    assert!(
        (vel.0 - expected).length() < 1e-3,
        "expected {expected:?}, got {:?}",
        vel.0
    );
}

// ── Behavior 25 — bolt at distance < MIN_PULL_DISTANCE clamps — PRESERVED

#[test]
fn pull_is_clamped_at_min_distance() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(1.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Distance floor = 20.0. Pull = 500 / 20 = 25 u/s². After 1s = 25 u/s.
    // Without the floor, pull at distance 1 would be ~500, which would be
    // a runaway.
    assert!(
        vel.0.length() < 100.0,
        "pull should be clamped (<100), got {}",
        vel.0.length()
    );
}

#[test]
fn pull_clamped_at_distance_one_produces_small_force() {
    // Production clamps distance via `.max(20.0)` in BOTH direction and
    // magnitude. At true distance=1 inside the clamp:
    //   delta = (-1, 0), distance_clamped = 20.0
    //   direction = (-1/20, 0) = (-0.05, 0)  (NOT unit-length)
    //   accel = direction * (500/20) = (-0.05, 0) * 25 = (-1.25, 0)
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(1.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - (-1.25)).abs() < 1e-3,
        "expected x = -1.25, got {}",
        vel.0.x
    );
    assert!(vel.0.y.abs() < 1e-5);
}

#[test]
fn pull_clamped_off_axis_keeps_direction_uncorrected() {
    // Edge: bolt at (0.5, 0.866) — direction NOT re-normalised after clamp.
    //   delta = (-0.5, -0.866), delta.length() ≈ 1.0 < 20 → clamp = 20
    //   direction = (-0.025, -0.0433)
    //   accel = direction * 25 = (-0.625, -1.0825) → after 1s, same.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(0.5, 0.866), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!((vel.0.length() - 1.25).abs() < 1e-3);
    assert!((vel.0.x - (-0.625)).abs() < 1e-2);
    assert!((vel.0.y - (-1.0825)).abs() < 1e-2);
}

// ── Behavior 26 — bolt at exactly MIN_PULL_DISTANCE → force = 25 u/s² ──

#[test]
fn pull_at_exact_min_distance_produces_twenty_five() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(20.0, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // At distance = 20, distance.max(20) = 20, force = 500/20 = 25.
    // Direction vector = (-20/20, 0) = (-1, 0) (unit-length).
    assert!((vel.0.x - (-25.0)).abs() < 1e-3);
    assert!(vel.0.y.abs() < 1e-5);
}

#[test]
fn pull_just_above_min_distance_uses_actual_distance() {
    // Edge: bolt at (20.0001, 0.0) — distance just above clamp; formula
    // uses actual distance → pull ≈ 500/20.0001 ≈ 24.9999.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(20.0001, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.x - (-24.9999)).abs() < 1e-2,
        "expected ≈ -24.9999, got {}",
        vel.0.x
    );
}

// ── Behavior 27 — bolt at same position as well → degenerate direction ──

#[test]
fn bolt_on_top_of_well_produces_zero_finite_force() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(vel.0.x.is_finite() && vel.0.y.is_finite());
    // delta = 0, distance_clamped = 20, direction = 0/20 = 0, accel = 0.
    assert!(vel.0.length() < 1e-4);
}

#[test]
fn bolt_essentially_at_well_produces_tiny_finite_force() {
    // Edge: bolt at (1e-9, 0) — tiny finite delta, tiny finite force.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::new(1e-9, 0.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(vel.0.x.is_finite() && vel.0.y.is_finite());
    assert!(vel.0.length() < 1e-6);
}

// ── Behavior 28 — two equidistant wells on X axis cancel — PRESERVED ───

#[test]
fn multiple_wells_compound_force() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    // Two wells equidistant on X axis → forces cancel in X.
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
    // Third well above the bolt → pull in +Y.
    spawn_well(&mut app, Vec2::new(0.0, 100.0), 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.x.abs() < 1e-4,
        "X-axis wells should cancel, got {}",
        vel.0.x
    );
    assert!(vel.0.y > 0.0, "+Y well should pull up, got {}", vel.0.y);
}

#[test]
fn two_wells_cancel_on_x_with_third_below_pulls_minus_y() {
    // Edge: two ±X wells plus one at (0,-100) below → X cancels, y = -5.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, -100.0), 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(vel.0.x.abs() < 1e-4);
    assert!((vel.0.y - (-5.0)).abs() < 1e-3);
}

// ── Behavior 29 — third well above bolt pulls in +Y — PRESERVED ───────

#[test]
fn three_wells_x_cancels_with_upper_well_pulling_positive_y() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, 100.0), 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(vel.0.x.abs() < 1e-4);
    assert!((vel.0.y - 5.0).abs() < 1e-3);
}

#[test]
fn three_wells_closer_above_well_below_pulls_minus_ten() {
    // Edge: move upper well to (0,-50) (below, closer) → y ≈ -10.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, -50.0), 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(vel.0.x.abs() < 1e-4);
    assert!((vel.0.y - (-10.0)).abs() < 1e-3);
}

// ── Behavior 30 — well with remaining <= 0 contributes NO force ────────

#[test]
fn expired_well_contributes_no_force_after_tick_filter() {
    // Both wells expire this tick (one was 0.05, decrements to -0.05;
    // other started at 0.0, decrements to -0.1). Neither passes the
    // positive-remaining filter → no force.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 0.05);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 0.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(vel.0.length() < 1e-4, "expected no force, got {:?}", vel.0);
}

#[test]
fn expired_wells_filtered_active_well_still_contributes() {
    // Edge: two expiring wells plus one surviving — only surviving well
    // pulls.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 0.05);
    spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 0.0);
    spawn_well(&mut app, Vec2::new(0.0, -100.0), 500.0, 2.0);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(vel.0.x.abs() < 1e-4);
    // Surviving well at (0,-100) pulls bolt at origin in -Y: accel = -5,
    // over dt=0.1 → velocity = -0.5.
    assert!((vel.0.y - (-0.5)).abs() < 1e-3);
}

// ── Behavior 31 — well.remaining ticks down by delta_secs — PRESERVED ──

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

// ── Behavior 32 — two bolts each get independent accumulation ──────────

#[test]
fn two_bolts_get_independent_pull_from_same_well() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt_a = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::new(0.0, 50.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let va = app.world().get::<Velocity2D>(bolt_a).unwrap().0;
    let vb = app.world().get::<Velocity2D>(bolt_b).unwrap().0;
    // bolt_a: distance 100, force 5, -X direction.
    assert!((va - Vec2::new(-5.0, 0.0)).length() < 1e-3);
    // bolt_b: distance 50, force 10, -Y direction.
    assert!((vb - Vec2::new(0.0, -10.0)).length() < 1e-3);
}

#[test]
fn third_bolt_gets_correct_directional_decomposition() {
    // Edge: bolt_c at (-50,-50), distance = 50*sqrt(2) ≈ 70.71.
    //   direction from bolt to well = (+1, +1)/sqrt(2) normalized
    //   accel magnitude = 500/70.71 ≈ 7.07, so per-axis ≈ 5.0.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let bolt_c = spawn_bolt(&mut app, Vec2::new(-50.0, -50.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let v = app.world().get::<Velocity2D>(bolt_c).unwrap().0;
    assert!((v.x - 5.0).abs() < 1e-2);
    assert!((v.y - 5.0).abs() < 1e-2);
}

// ── Behavior 33 — zero wells in world → no velocity change ────────────

#[test]
fn zero_wells_in_world_no_velocity_change() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(3.0, 4.0));
}

#[test]
fn zero_wells_second_tick_velocity_unchanged() {
    // Edge: second tick — still bitwise unchanged.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0), Vec2::new(3.0, 4.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(3.0, 4.0));
}

// ── Behavior 34 — zero bolts → no panic; wells still tick ────────────

#[test]
fn zero_bolts_in_world_no_panic_wells_still_tick() {
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

    let remaining = app.world().get::<GravityWell>(well).unwrap().remaining;
    assert!((remaining - 1.5).abs() < 1e-5);
}

#[test]
fn non_bolt_entity_is_excluded_from_pull() {
    // Edge: non-Bolt entity with Position2D + Velocity2D → unchanged by
    // the `With<Bolt>` filter.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
    let non_bolt = app
        .world_mut()
        .spawn((Position2D(Vec2::ZERO), Velocity2D(Vec2::new(5.0, 5.0))))
        .id();

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let vel = app.world().get::<Velocity2D>(non_bolt).unwrap();
    assert_eq!(vel.0, Vec2::new(5.0, 5.0));
}

// ── Behavior 35 — same-tick multi-well multi-bolt with filter ─────────

#[test]
fn multi_well_multi_bolt_with_expired_wells_filtered() {
    // Four wells: two active (±Y at ±100), two expiring (at ±X).
    // Two bolts. After the tick's snapshot filter, only the ±Y wells
    // contribute.
    let mut app = test_app_playing();
    wire_pull_only(&mut app);
    spawn_well(&mut app, Vec2::new(0.0, 100.0), 200.0, 2.0);
    spawn_well(&mut app, Vec2::new(0.0, -100.0), 200.0, 2.0);
    spawn_well(&mut app, Vec2::new(50.0, 0.0), 200.0, 0.0);
    spawn_well(&mut app, Vec2::new(-50.0, 0.0), 200.0, 0.05);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::new(10.0, 10.0), Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // bolt_a at origin: ±Y wells cancel in Y (they pull toward +Y and -Y
    // symmetrically), both on X axis = 0 → no X either.
    let va = app.world().get::<Velocity2D>(bolt_a).unwrap().0;
    assert!(
        va.length() < 1e-3,
        "bolt_a should have ~zero force, got {va:?}"
    );

    // bolt_b at (10,10): both surviving wells pull.
    let vb = app.world().get::<Velocity2D>(bolt_b).unwrap().0;
    assert!(vb.length() > 0.0, "bolt_b should feel force");
    assert!(
        vb.length() < 10.0,
        "bolt_b force should be modest, got {vb:?}"
    );
}

#[test]
fn multi_well_multi_bolt_preserves_all_four_entities_this_frame() {
    // Edge: despawn is NOT wired — all four wells still exist.
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
