//! Group E — `afterimage_check_phantom_bounce` (Behaviors CB1–CB10).
//!
//! Pins the physical-overlap reflection system:
//! - Bolt overlapping a `PhantomBreaker` AABB + moving down → vertical-mirror
//!   velocity, snap position to top face + `BoltRadius`, emit exactly one
//!   `BumpPerformed { bolt: Some(b), breaker: phantom_entity, grade }`.
//! - Bolt not overlapping → no reflection, no `BumpPerformed`.
//! - Entry guard: bolt moving UP through the AABB → no reflection, no
//!   `BumpPerformed` (prevents re-bouncing a bolt that has already cleared
//!   the phantom).
//! - Grade forwarding: `Perfect` when real breaker's `BumpState.active &&
//!   timer <= perfect_window`; `Early` when `active && timer >
//!   perfect_window`; `Late` via the retroactive path when `!active` and
//!   `retroactive_grade(...)` returns `Late`. Grade is pinned through the
//!   emitted `BumpPerformed.grade` field ONLY — writer-tests MUST NOT
//!   call `retroactive_grade` directly (it is `pub(super)` in `bump` module
//!   and not reachable from this file; the test is a PURE emitted-grade
//!   assertion).
//! - `Without<PhantomBolt>` filter prevents cascade re-bounces from
//!   spawned phantom bolts.
//! - `bounced_this_frame` dedup guarantees ONE emission per bolt per frame
//!   even when the bolt overlaps multiple `PhantomBreakers`.
//! - No `PhantomBreaker` / no `Breaker` cases are no-op early returns.

use bevy::prelude::*;

use super::{
    super::system::{PhantomBreaker, PhantomBreakerLifetime},
    helpers::{
        build_afterimage_app, captured_bump_performed, seed_active_protocols_with_afterimage,
        spawn_breaker_with_bump_state, spawn_phantom_bolt_entity, spawn_phantom_breaker_at,
        spawn_real_bolt,
    },
};
use crate::{
    breaker::{
        components::{BaseHeight, BaseWidth, BumpState},
        messages::BumpGrade,
    },
    prelude::*,
};

// ── Canonical phantom-breaker factory ───────────────────────────────────────
//
// `PhantomBreaker` at origin with canonical `BaseWidth(100.0)` and
// `BaseHeight(20.0)` → AABB x ∈ [-50.0, 50.0], y ∈ [-10.0, 10.0]. Top face
// at y = 10.0.
fn spawn_canonical_phantom_breaker(app: &mut App) -> Entity {
    // Helper already installs BaseWidth(100.0) / BaseHeight(20.0) and
    // CleanupOnExit. Override lifetime if callers need it; default 1.5s.
    spawn_phantom_breaker_at(app, Vec2::ZERO, 1.5)
}

fn perfect_bump_state() -> BumpState {
    BumpState {
        active:         true,
        timer:          0.1, // <= perfect_window 0.2 → Perfect
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    }
}

fn early_bump_state() -> BumpState {
    BumpState {
        active:         true,
        timer:          0.3, // > perfect_window 0.2 → Early
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    }
}

// ── CB1 — bolt overlapping + down → mirror, reposition, emit one bump ─────

#[test]
fn bolt_overlapping_and_moving_down_reflects_and_emits_one_bump_performed() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        // Bolt center at (0, 14) — 4 units inside the top face at y=10.
        // With radius 6 the bolt overlaps the AABB (bottom of bolt at y=8).
        // Harness has no velocity integration, so the bolt must be placed
        // already overlapping at spawn time for the overlap check to fire.
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("bolt velocity must exist after tick");
    assert!(
        velocity.0.y > 0.0,
        "bolt velocity.y must be reflected upward after overlap, got {}",
        velocity.0.y
    );
    assert!(
        (velocity.0.y - 400.0).abs() < 1.0,
        "reflected |velocity.y| must be ~400.0, got {}",
        velocity.0.y
    );
    assert!(
        velocity.0.x.abs() < f32::EPSILON,
        "x component must be preserved at 0.0, got {}",
        velocity.0.x
    );

    let position = app
        .world()
        .get::<Position2D>(bolt)
        .expect("bolt position must exist after tick");
    assert!(
        position.0.y <= 10.0 + 6.0 + 0.01 && position.0.y >= 10.0 + 6.0 - 0.01,
        "bolt must be repositioned to top face (10.0) + radius (6.0) = 16.0, got y={}",
        position.0.y
    );

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "exactly ONE BumpPerformed must be emitted on overlap, got {}",
        bumps.len()
    );
    assert_eq!(bumps[0].bolt, Some(bolt));
    assert_eq!(
        bumps[0].breaker, phantom,
        "BumpPerformed.breaker must be the PHANTOM entity (not the real breaker)"
    );
    assert_eq!(bumps[0].grade, BumpGrade::Perfect);
}

// ── CB1 (edge case) — deeply overlapping bolt still produces one reflect ──

#[test]
fn deeply_overlapping_bolt_still_reflects_exactly_once() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 5.0), // deeply inside the AABB
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "a deeply overlapping bolt moving down must still emit EXACTLY one BumpPerformed, got {}",
        bumps.len()
    );
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "bolt must be reflected upward, got velocity.y = {}",
        velocity.0.y
    );
}

// ── CB2 — bolt NOT overlapping → no reflection, no BumpPerformed ──────────

#[test]
fn bolt_not_overlapping_phantom_does_not_reflect_or_emit() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(500.0, 500.0), // well outside AABB
        Vec2::new(0.0, 400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - 400.0).abs() < 1.0,
        "bolt velocity must be UNCHANGED when no overlap, got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "zero BumpPerformed messages expected when bolt does not overlap"
    );
}

// ── CB2 (edge case) — bolt aligned on y but x-outside does not overlap ────

#[test]
fn bolt_aligned_on_y_but_x_outside_does_not_overlap() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let _bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(200.0, 0.0), // x outside half-width 50
        Vec2::new(0.0, 400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    assert!(
        captured_bump_performed(&app).is_empty(),
        "bolt aligned on y but x-outside AABB must NOT emit BumpPerformed"
    );
}

// ── CB3 — bolt overlapping but moving UP → no reflection (entry guard) ────

#[test]
fn bolt_overlapping_but_moving_up_does_not_reflect() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 5.0),   // inside AABB above center
        Vec2::new(0.0, 400.0), // moving UP
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - 400.0).abs() < 1.0,
        "upward-moving bolt must NOT be reflected (entry guard), got velocity.y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "upward-moving bolt must NOT emit BumpPerformed"
    );
}

// ── CB3 (edge case) — same geometry moving DOWN reflects + emits ──────────

#[test]
fn same_geometry_moving_down_does_reflect_and_emit() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 5.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "downward-moving bolt inside AABB MUST be reflected, got velocity.y={}",
        velocity.0.y
    );
    assert_eq!(captured_bump_performed(&app).len(), 1);
}

// ── CB4 — grade Perfect when active && timer <= perfect_window ────────────

#[test]
fn bump_grade_is_perfect_when_active_and_timer_within_perfect_window() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0), // overlapping top face (4 units inside, radius 6)
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(bumps[0].grade, BumpGrade::Perfect);
    assert_eq!(bumps[0].bolt, Some(bolt));
    assert_eq!(bumps[0].breaker, phantom);
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.y > 0.0 && (velocity.0.y - 400.0).abs() < 1.0,
        "bolt vertical-mirror reflect: expected ~+400.0, got {}",
        velocity.0.y
    );
    assert!(
        velocity.0.x.abs() < f32::EPSILON,
        "x component preserved at 0.0, got {}",
        velocity.0.x
    );
}

// ── CB4 (edge case) — timer exactly at boundary still Perfect (inclusive) ─

#[test]
fn bump_grade_is_perfect_when_timer_exactly_at_perfect_window_boundary() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let boundary_state = BumpState {
        active:         true,
        timer:          0.2, // exactly at perfect_window 0.2
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    };
    let _breaker = spawn_breaker_with_bump_state(&mut app, boundary_state, 0.2, 0.15);
    let _bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(
        bumps[0].grade,
        BumpGrade::Perfect,
        "timer == perfect_window must still be Perfect (inclusive <=)"
    );
}

// ── CB5 — grade Early when active && timer > perfect_window ───────────────

#[test]
fn bump_grade_is_early_when_active_and_timer_exceeds_perfect_window() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, early_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(bumps[0].grade, BumpGrade::Early);
    assert_eq!(bumps[0].bolt, Some(bolt));
    assert_eq!(bumps[0].breaker, phantom);
}

// ── CB5 (edge case) — timer just above boundary is Early (strict >) ───────

#[test]
fn bump_grade_is_early_when_timer_just_above_perfect_window() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let above_state = BumpState {
        active:         true,
        timer:          0.2001, // just above perfect_window 0.2
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    };
    let _breaker = spawn_breaker_with_bump_state(&mut app, above_state, 0.2, 0.15);
    let _bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(bumps[0].grade, BumpGrade::Early);
}

// ── CB6 — grade Late via retroactive path ─────────────────────────────────

#[test]
fn bump_grade_is_late_via_retroactive_path() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_canonical_phantom_breaker(&mut app);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    // post_hit_timer = 0.05 → time_since_hit = (0.2 + 0.15) - 0.05 = 0.30
    // 0.30 > 0.2 perfect_window → retroactive_grade returns Late.
    let retroactive_late = BumpState {
        active:         false,
        timer:          0.0,
        post_hit_timer: 0.05,
        cooldown:       0.0,
        last_hit_bolt:  Some(bolt),
    };
    let _breaker = spawn_breaker_with_bump_state(&mut app, retroactive_late, 0.2, 0.15);

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(
        bumps[0].grade,
        BumpGrade::Late,
        "retroactive path with time_since_hit > perfect_window must emit Late"
    );
    assert_eq!(bumps[0].bolt, Some(bolt));
    assert_eq!(bumps[0].breaker, phantom);
}

// ── CB6 (edge case) — retroactive path inside perfect window → Perfect ────

#[test]
fn retroactive_path_inside_perfect_window_emits_perfect() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    // post_hit_timer = 0.34 → time_since_hit = (0.2 + 0.15) - 0.34 = 0.01
    // 0.01 <= 0.2 → retroactive_grade returns Perfect.
    let retroactive_perfect = BumpState {
        active:         false,
        timer:          0.0,
        post_hit_timer: 0.34,
        cooldown:       0.0,
        last_hit_bolt:  Some(bolt),
    };
    let _breaker = spawn_breaker_with_bump_state(&mut app, retroactive_perfect, 0.2, 0.15);

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(bumps.len(), 1);
    assert_eq!(
        bumps[0].grade,
        BumpGrade::Perfect,
        "retroactive path with time_since_hit <= perfect_window must emit Perfect"
    );
}

// ── CB7 — Without<PhantomBolt> filter prevents cascade ─────────────────────

#[test]
fn phantom_bolt_filter_prevents_cascade_re_bounce() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom_breaker = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    // Spawn a PHANTOM bolt OVERLAPPING the phantom breaker AABB (center
    // inside the top-face) moving DOWN — identical geometry to CB1 so the
    // only thing preventing reflection is the `Without<PhantomBolt>`
    // filter.
    let dummy = app.world_mut().spawn_empty().id();
    let phantom_bolt = spawn_phantom_bolt_entity(
        &mut app,
        dummy,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        3.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom_bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "phantom bolt must NOT be reflected (Without<PhantomBolt> filter), got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "cascade prevention: phantom bolt overlapping phantom breaker must emit zero BumpPerformed"
    );
}

// ── CB7 (edge case) — stale PhantomOwner does not panic ───────────────────

#[test]
fn phantom_bolt_with_stale_owner_does_not_panic() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom_breaker = spawn_canonical_phantom_breaker(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    // Spawn a dummy, then IMMEDIATELY despawn it — the PhantomOwner points
    // at a dead entity.
    let dummy = app.world_mut().spawn_empty().id();
    let phantom_bolt = spawn_phantom_bolt_entity(
        &mut app,
        dummy,
        Vec2::new(0.0, -16.0),
        Vec2::new(0.0, 400.0),
        3.0,
    );
    app.world_mut().entity_mut(dummy).despawn();

    tick(&mut app);

    // No panic. Phantom still alive (hasn't expired).
    assert!(
        app.world().get_entity(phantom_bolt).is_ok(),
        "phantom bolt must survive even with stale PhantomOwner"
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "cascade prevention holds even with stale PhantomOwner"
    );
}

// ── CB8 — bounced_this_frame dedup: bolt overlapping two phantoms ─────────

#[test]
fn bounced_this_frame_dedup_emits_exactly_one_bump_even_when_two_phantoms_overlap() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    // Two PhantomBreakers — their AABBs overlap each other, and both
    // contain the origin.
    let phantom_a = app
        .world_mut()
        .spawn((
            PhantomBreaker,
            PhantomBreakerLifetime(1.5),
            Position2D(Vec2::new(-10.0, 0.0)),
            BaseWidth(100.0),
            BaseHeight(20.0),
        ))
        .id();
    let phantom_b = app
        .world_mut()
        .spawn((
            PhantomBreaker,
            PhantomBreakerLifetime(1.5),
            Position2D(Vec2::new(10.0, 0.0)),
            BaseWidth(100.0),
            BaseHeight(20.0),
        ))
        .id();

    // Entry-guard requires downward motion: bolt ABOVE the phantom AABB
    // moving DOWN into it.
    let real_bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "EXACTLY one BumpPerformed must be emitted even when the bolt overlaps two \
         PhantomBreakers — dedup must coalesce, got {}",
        bumps.len()
    );
    assert_eq!(bumps[0].bolt, Some(real_bolt));
    assert!(
        bumps[0].breaker == phantom_a || bumps[0].breaker == phantom_b,
        "emitted breaker must be one of the two phantoms"
    );
    let velocity = app.world().get::<Velocity2D>(real_bolt).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "bolt must be reflected ONCE, got velocity.y={}",
        velocity.0.y
    );
}

// ── CB8 (edge case) — three overlapping phantoms still emit exactly one ───

#[test]
fn three_overlapping_phantoms_emit_exactly_one_bump_performed() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    for offset in [-10.0, 0.0, 10.0] {
        app.world_mut().spawn((
            PhantomBreaker,
            PhantomBreakerLifetime(1.5),
            Position2D(Vec2::new(offset, 0.0)),
            BaseWidth(100.0),
            BaseHeight(20.0),
        ));
    }

    let _real_bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    tick(&mut app);

    assert_eq!(
        captured_bump_performed(&app).len(),
        1,
        "EXACTLY one BumpPerformed even with three overlapping phantoms"
    );
}

// ── CB9 — no PhantomBreaker in world → no-op ──────────────────────────────

#[test]
fn zero_phantom_breakers_in_world_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "no PhantomBreaker in world → bolt velocity unchanged, got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "no PhantomBreaker in world → zero BumpPerformed emitted"
    );
}

// ── CB9 (edge case) — zero bolts AND zero phantoms = quiet tick ───────────

#[test]
fn zero_phantoms_and_zero_bolts_is_quiet() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    tick(&mut app);

    assert!(captured_bump_performed(&app).is_empty());
}

// ── CB10 — no real Breaker in world → early-return ────────────────────────

#[test]
fn zero_real_breakers_in_world_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );
    // No real breaker spawned — `breaker_query.single()` should fail.

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.y - (-400.0)).abs() < 1.0,
        "no real Breaker → bolt velocity unchanged, got y={}",
        velocity.0.y
    );
    assert!(
        captured_bump_performed(&app).is_empty(),
        "no real Breaker → zero BumpPerformed emitted"
    );
}

// ── Anchor multiplier widens effective perfect window for phantom bounce ─

#[test]
fn anchor_planted_and_active_scales_perfect_window_for_phantom_bounce_grade() {
    // Given: a real breaker with BumpPerfectWindow(0.1), BumpLateWindow(0.2),
    // and BumpState { active: true, timer: 0.15 } — `timer` is JUST past the
    // raw perfect window (0.15 > 0.1), so the forward-grade would be `Early`
    // without anchor widening. Attach AnchorPlanted + AnchorActive with
    // perfect_window_multiplier: 2.0 so effective_pw = 0.1 * 2.0 = 0.2,
    // putting `timer` (0.15) back inside the widened perfect window.
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom = spawn_canonical_phantom_breaker(&mut app);

    let widened_state = BumpState {
        active:         true,
        timer:          0.15, // > raw 0.1 (Early) but <= widened 0.2 (Perfect)
        post_hit_timer: 0.0,
        cooldown:       0.0,
        last_hit_bolt:  None,
    };
    let breaker = spawn_breaker_with_bump_state(&mut app, widened_state, 0.1, 0.2);
    app.world_mut().entity_mut(breaker).insert((
        AnchorPlanted,
        AnchorActive {
            bump_force_multiplier:     1.5,
            perfect_window_multiplier: 2.0,
            plant_delay:               0.3,
        },
    ));

    // Bolt overlapping the phantom AABB, moving DOWN into it.
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    let bumps = captured_bump_performed(&app);
    assert_eq!(
        bumps.len(),
        1,
        "exactly one BumpPerformed expected on overlap, got {}",
        bumps.len()
    );
    assert_eq!(bumps[0].bolt, Some(bolt));
    assert_eq!(bumps[0].breaker, phantom);
    assert_eq!(
        bumps[0].grade,
        BumpGrade::Perfect,
        "AnchorPlanted + AnchorActive(perfect_window_multiplier: 2.0) must \
         widen effective_pw from 0.1 to 0.2, promoting timer=0.15 from Early \
         to Perfect"
    );
}

// ── Regression — horizontal bolt tangentially overlapping phantom no-ops ──
//
// Off-by-one regression in the entry guard: prior to the fix, the check
// read `velocity.y > 0.0` instead of `velocity.y >= 0.0`. A bolt with
// purely horizontal motion (velocity.y == 0.0) that happened to tangentially
// overlap a phantom AABB would fall through the guard, emit a spurious
// `BumpPerformed`, mirror velocity.y (a no-op for 0.0), and snap the bolt's
// y position to the top face + radius. Horizontal tangential contact is
// not a bump — the guard must treat `velocity.y == 0.0` the same as
// `velocity.y > 0.0`. This test pins the inclusive-zero behaviour.

#[test]
fn horizontal_velocity_tangentially_overlapping_phantom_does_not_emit_bump_performed() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);

    // PhantomBreaker at (0.0, -50.0) with half-extents (30.0, 5.0).
    // AABB: x ∈ [-30.0, 30.0], y ∈ [-55.0, -45.0] — top face at y = -45.0.
    app.world_mut().spawn((
        PhantomBreaker,
        PhantomBreakerLifetime(1.5),
        Position2D(Vec2::new(0.0, -50.0)),
        BaseWidth(60.0),
        BaseHeight(10.0),
    ));

    let _breaker = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);

    // Bolt center 1 unit above the phantom top face (y = -44.0 vs top = -45.0),
    // radius 8.0 → circle bottom at y = -52.0 tangentially overlaps the AABB.
    // Velocity is PURELY HORIZONTAL (velocity.y == 0.0).
    let bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, -44.0),
        Vec2::new(300.0, 0.0),
        10.0,
        8.0,
    );

    tick(&mut app);

    // Guard must skip — no BumpPerformed.
    assert!(
        captured_bump_performed(&app).is_empty(),
        "horizontal bolt (velocity.y == 0.0) tangentially overlapping phantom must NOT emit \
         BumpPerformed — the entry guard must use `velocity.y >= 0.0` (inclusive), got {} emissions",
        captured_bump_performed(&app).len(),
    );

    // Velocity must be unchanged — no mirror, no snap.
    let velocity = app
        .world()
        .get::<Velocity2D>(bolt)
        .expect("bolt velocity must exist after tick");
    assert_eq!(
        velocity.0,
        Vec2::new(300.0, 0.0),
        "horizontal bolt velocity must be UNCHANGED (no reflection, no mirror), got {:?}",
        velocity.0,
    );

    // Position y must be unchanged — the system must not have snapped the
    // bolt to top-face + radius.
    let position = app
        .world()
        .get::<Position2D>(bolt)
        .expect("bolt position must exist after tick");
    assert!(
        (position.0.y - (-44.0)).abs() < f32::EPSILON,
        "bolt position y must be UNCHANGED (no snap), expected -44.0, got {}",
        position.0.y,
    );
}

// ── CB10 (edge case) — TWO real breakers still triggers early-return ──────

#[test]
fn two_real_breakers_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let _phantom = spawn_canonical_phantom_breaker(&mut app);
    let _breaker_a = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let _breaker_b = spawn_breaker_with_bump_state(&mut app, perfect_bump_state(), 0.2, 0.15);
    let _bolt = spawn_real_bolt(
        &mut app,
        Vec2::new(0.0, 14.0),
        Vec2::new(0.0, -400.0),
        10.0,
        6.0,
    );

    tick(&mut app);

    assert!(
        captured_bump_performed(&app).is_empty(),
        "two real breakers → breaker_query.single() fails → no BumpPerformed"
    );
}
