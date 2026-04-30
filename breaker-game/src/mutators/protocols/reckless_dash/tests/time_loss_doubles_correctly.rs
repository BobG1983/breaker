//! Group C — `double_behavior` float arithmetic and saturating-mul edge cases
//! (Behaviors 16–20).
//!
//! Pins the semantics of the `double_behavior` helper through end-to-end system
//! mutations:
//! - `TimeLoss(d)` → `TimeLoss(d * 2.0)` (float doubling, including zero and
//!   negative)
//! - `BoltLossBehavior::None` → `BoltLossBehavior::None` (uniform handling)
//! - `LifeLoss(n)` → `LifeLoss(n.saturating_mul(2))` (overflow safety)
//! - End-to-end: `BoltLost` during dash with `TimeLoss(5.0)` writes one
//!   `ReduceNodeTimer { delta: 10.0 }`

use bevy::prelude::*;

use super::{
    super::system::OriginalBoltLossBehavior,
    helpers::{
        build_reckless_dash_e2e_app, build_reckless_dash_transition_app,
        captured_reduce_node_timer, force_dash_state, seed_active_protocols_with_reckless_dash,
        spawn_breaker_for_e2e, spawn_breaker_for_transition, write_bolt_lost,
    },
};
use crate::{
    breaker::components::{BoltLossBehavior, DashState},
    prelude::*,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}

// ── Behavior 16 — TimeLoss doubles to the correct float value on dash enter ──

#[test]
fn timeloss_five_doubles_to_ten_on_dash_enter() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::TimeLoss(5.0),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    let BoltLossBehavior::TimeLoss(actual) = live else {
        panic!("expected TimeLoss variant, got {live:?}");
    };
    assert!(
        (actual - 10.0).abs() < f32::EPSILON,
        "TimeLoss(5.0) must double to TimeLoss(10.0), got TimeLoss({actual})"
    );
    assert_eq!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .map(|o| o.0),
        Some(BoltLossBehavior::TimeLoss(5.0)),
        "OriginalBoltLossBehavior must carry TimeLoss(5.0)"
    );
}

// ── Behavior 16 (edge case) — TimeLoss(0.5) doubles to TimeLoss(1.0) ─────────

#[test]
fn timeloss_half_doubles_to_one() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::TimeLoss(0.5),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    let BoltLossBehavior::TimeLoss(actual) = live else {
        panic!("expected TimeLoss variant, got {live:?}");
    };
    assert!(
        (actual - 1.0).abs() < f32::EPSILON,
        "TimeLoss(0.5) must double to TimeLoss(1.0), got TimeLoss({actual})"
    );
}

// ── Behavior 16 (edge case) — TimeLoss(0.0) stays TimeLoss(0.0) (no NaN) ────

#[test]
fn timeloss_zero_doubles_to_zero_no_nan() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::TimeLoss(0.0),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    let BoltLossBehavior::TimeLoss(actual) = live else {
        panic!("expected TimeLoss variant, got {live:?}");
    };
    assert!(
        actual == 0.0 && !actual.is_nan(),
        "TimeLoss(0.0) must double to TimeLoss(0.0) with no NaN, got TimeLoss({actual})"
    );
}

// ── Behavior 16 (edge case) — TimeLoss(-3.0) doubles to TimeLoss(-6.0) ───────
// Design intent: double_behavior performs raw *2.0; clamping is the caller's
// responsibility elsewhere. Negative TimeLoss is not valid game-design but the
// helper must not panic or produce unexpected values.

#[test]
fn timeloss_negative_doubles_raw_no_clamp() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::TimeLoss(-3.0),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    let BoltLossBehavior::TimeLoss(actual) = live else {
        panic!("expected TimeLoss variant, got {live:?}");
    };
    assert!(
        (actual - (-6.0)).abs() < f32::EPSILON,
        "TimeLoss(-3.0) must double to TimeLoss(-6.0) (raw *2.0, no clamp), got TimeLoss({actual})"
    );
}

// ── Behavior 17 — BoltLost while Dashing with TimeLoss(5.0) writes ReduceNodeTimer { delta: 10.0 } ─

#[test]
fn bolt_lost_during_dash_with_timeloss_writes_reduce_node_timer_delta_ten() {
    let mut app = build_reckless_dash_e2e_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::TimeLoss(5.0),
        None,
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // Same tick: force dash enter + write BoltLost.
    // reckless_dash_on_dash_transition doubles TimeLoss(5.0) → TimeLoss(10.0)
    // BEFORE handle_bolt_lost reads the message.
    force_dash_state(&mut app, breaker, DashState::Dashing);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let msgs = captured_reduce_node_timer(&app);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one ReduceNodeTimer must be written on a BoltLost with TimeLoss"
    );
    assert!(
        (msgs[0].delta - 10.0).abs() < f32::EPSILON,
        "ReduceNodeTimer delta must be 10.0 (doubled from 5.0), got {}",
        msgs[0].delta
    );
}

// ── Behavior 18 — BoltLossBehavior::None doubles to BoltLossBehavior::None ──

#[test]
fn none_doubles_to_none_on_dash_enter() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::None,
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::None,
        "BoltLossBehavior::None must double to BoltLossBehavior::None"
    );
    assert_eq!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .map(|o| o.0),
        Some(BoltLossBehavior::None),
        "OriginalBoltLossBehavior(None) overlay must be inserted even when behavior is None"
    );
}

// ── Behavior 18 (edge case) — None restored from overlay on dash exit ────────

#[test]
fn none_restored_from_overlay_on_dash_exit() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    // Manually construct the mid-dash state with None behavior and overlay.
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::None,
        DashState::Dashing,
        DashState::Dashing,
    );
    app.world_mut()
        .entity_mut(breaker)
        .insert(OriginalBoltLossBehavior(BoltLossBehavior::None));

    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::None,
        "BoltLossBehavior must be restored to None from overlay on dash exit"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "OriginalBoltLossBehavior overlay must be removed on dash exit"
    );
}

// ── Behavior 19 — LifeLoss(0) doubles to LifeLoss(0) via saturating_mul ──────

#[test]
fn lifeloss_zero_doubles_to_zero_via_saturating_mul() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(0),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(0),
        "LifeLoss(0).saturating_mul(2) must equal LifeLoss(0)"
    );
}

// ── Behavior 19 (edge case) — LifeLoss(0) restored from overlay on dash exit ─

#[test]
fn lifeloss_zero_restored_from_overlay_on_dash_exit() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(0),
        DashState::Dashing,
        DashState::Dashing,
    );
    app.world_mut()
        .entity_mut(breaker)
        .insert(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(0)));

    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(0),
        "LifeLoss(0) must be restored from overlay on dash exit"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none()
    );
}

// ── Behavior 20 — LifeLoss(u32::MAX) saturates to LifeLoss(u32::MAX) (no panic) ─

#[test]
fn lifeloss_u32_max_saturates_to_u32_max_no_panic() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(u32::MAX),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app); // must not panic

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(u32::MAX),
        "LifeLoss(u32::MAX).saturating_mul(2) must equal LifeLoss(u32::MAX)"
    );
}

// ── Behavior 20 (edge case) — LifeLoss(u32::MAX / 2 + 1) saturates to u32::MAX ─

#[test]
fn lifeloss_above_half_max_saturates_to_u32_max() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(u32::MAX / 2 + 1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(u32::MAX),
        "LifeLoss(u32::MAX/2+1).saturating_mul(2) must saturate to LifeLoss(u32::MAX)"
    );
}

// ── Behavior 20 (edge case) — LifeLoss(u32::MAX / 2) does NOT saturate ───────

#[test]
fn lifeloss_exactly_half_max_doubles_without_saturation() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let n = u32::MAX / 2;
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(n),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    // u32::MAX is 4294967295; u32::MAX / 2 = 2147483647; * 2 = 4294967294 = u32::MAX - 1.
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(u32::MAX - 1),
        "LifeLoss(u32::MAX/2).saturating_mul(2) must equal LifeLoss(u32::MAX-1), not saturate"
    );
}
