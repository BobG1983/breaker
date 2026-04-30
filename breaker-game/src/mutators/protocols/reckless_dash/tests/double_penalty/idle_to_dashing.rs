use super::{
    super::{
        super::system::OriginalBoltLossBehavior,
        helpers::{
            build_reckless_dash_transition_app, force_dash_state, spawn_breaker_for_transition,
        },
    },
    helpers::seed_canonical,
};
use crate::{
    breaker::components::{BoltLossBehavior, DashState},
    prelude::*,
};

// ── Behavior 1 — Idle → Dashing inserts OriginalBoltLossBehavior overlay ───

#[test]
fn idle_to_dashing_inserts_overlay_with_original_lifeloss_one() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let overlay = app
        .world()
        .get::<OriginalBoltLossBehavior>(breaker)
        .map(|o| o.0);
    assert_eq!(
        overlay,
        Some(BoltLossBehavior::LifeLoss(1)),
        "Idle → Dashing must insert OriginalBoltLossBehavior(LifeLoss(1))"
    );
}

// ── Behavior 1 (edge case) — TimeLoss overlay carries original TimeLoss value ─

#[test]
fn idle_to_dashing_inserts_overlay_with_original_timeloss_five() {
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

    let overlay = app
        .world()
        .get::<OriginalBoltLossBehavior>(breaker)
        .map(|o| o.0);
    assert_eq!(
        overlay,
        Some(BoltLossBehavior::TimeLoss(5.0)),
        "Idle → Dashing must insert OriginalBoltLossBehavior(TimeLoss(5.0))"
    );
}

// ── Behavior 2 — Idle → Dashing doubles the live BoltLossBehavior ──────────

#[test]
fn idle_to_dashing_doubles_lifeloss_one_to_two() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
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
        BoltLossBehavior::LifeLoss(2),
        "Idle → Dashing must double BoltLossBehavior::LifeLoss(1) to LifeLoss(2)"
    );
}

// ── Behavior 2 (edge case) — TimeLoss(5.0) doubles to TimeLoss(10.0) ────────

#[test]
fn idle_to_dashing_doubles_timeloss_five_to_ten() {
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
}
