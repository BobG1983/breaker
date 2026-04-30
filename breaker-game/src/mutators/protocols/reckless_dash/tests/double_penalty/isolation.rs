use super::{
    super::{
        super::system::OriginalBoltLossBehavior,
        helpers::{
            build_reckless_dash_transition_app, force_dash_state, original_behavior,
            spawn_breaker_for_transition,
        },
    },
    helpers::seed_canonical,
};
use crate::{
    breaker::components::{BoltLossBehavior, DashState},
    prelude::*,
};

// ── Behavior 10 — protocol_active gate excludes system when inactive ─────────

#[test]
fn system_skipped_when_reckless_dash_not_in_active_protocols() {
    let mut app = build_reckless_dash_transition_app();
    // Do NOT seed ActiveProtocols — protocol_active(RecklessDash) returns false.

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
        BoltLossBehavior::LifeLoss(1),
        "system must not run when Reckless Dash is not active"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "no overlay must be inserted when system is gated off"
    );
}

// ── Behavior 11 — multi-breaker isolation ───────────────────────────────────

#[test]
fn only_transitioning_breaker_is_mutated_other_breaker_unchanged() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    let breaker_a = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );
    let breaker_b = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    // Only breaker A transitions — breaker B's DashState is never mutated.
    force_dash_state(&mut app, breaker_a, DashState::Dashing);
    tick(&mut app);

    // Breaker A: doubled and overlay inserted.
    let live_a = *app
        .world()
        .get::<BoltLossBehavior>(breaker_a)
        .expect("breaker_a must have BoltLossBehavior");
    assert_eq!(
        live_a,
        BoltLossBehavior::LifeLoss(2),
        "breaker_a must be doubled to LifeLoss(2)"
    );
    assert_eq!(
        original_behavior(&app, breaker_a),
        Some(BoltLossBehavior::LifeLoss(1)),
        "breaker_a must have OriginalBoltLossBehavior(LifeLoss(1))"
    );

    // Breaker B: completely unchanged (Changed<DashState> was false for it).
    let live_b = *app
        .world()
        .get::<BoltLossBehavior>(breaker_b)
        .expect("breaker_b must have BoltLossBehavior");
    assert_eq!(
        live_b,
        BoltLossBehavior::LifeLoss(1),
        "breaker_b must remain LifeLoss(1) — its DashState did not change"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker_b)
            .is_none(),
        "breaker_b must have no OriginalBoltLossBehavior component"
    );
}
