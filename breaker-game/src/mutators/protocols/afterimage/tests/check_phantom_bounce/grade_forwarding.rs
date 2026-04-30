use bevy::prelude::*;

use super::{
    super::helpers::{
        build_afterimage_app, captured_bump_performed, seed_active_protocols_with_afterimage,
        spawn_breaker_with_bump_state, spawn_real_bolt,
    },
    early_bump_state, perfect_bump_state, spawn_canonical_phantom_breaker,
};
use crate::{
    breaker::{components::BumpState, messages::BumpGrade},
    prelude::*,
};

// ── CB3 — entry guard: bolt moving UP does not reflect ────────────────────

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
