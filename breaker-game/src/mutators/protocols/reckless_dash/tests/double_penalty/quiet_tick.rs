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

// ── Behavior 5 — mid-dash tick (no Changed<DashState>) is a no-op ────────────

#[test]
fn mid_dash_tick_without_state_change_leaves_behavior_unchanged() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    // Spawn already-dashing breaker; do NOT force a state mutation.
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(2),
        DashState::Dashing,
        DashState::Dashing,
    );

    // No force_dash_state call — Changed<DashState> is false.
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(2),
        "mid-dash quiet tick must not change BoltLossBehavior"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "no overlay must be inserted on a mid-dash quiet tick"
    );
}

// ── Behavior 5 (edge case) — overlay present, mid-dash quiet tick ────────────

#[test]
fn mid_dash_tick_with_overlay_present_leaves_overlay_unchanged() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(2),
        DashState::Dashing,
        DashState::Dashing,
    );
    app.world_mut()
        .entity_mut(breaker)
        .insert(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));

    // No force_dash_state call.
    tick(&mut app);

    assert_eq!(
        original_behavior(&app, breaker),
        Some(BoltLossBehavior::LifeLoss(1)),
        "mid-dash quiet tick must leave OriginalBoltLossBehavior overlay intact"
    );
    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(2),
        "mid-dash quiet tick must not change live BoltLossBehavior"
    );
}

// ── Behavior 6 — Idle → Idle (same-value write, Changed fires) is a no-op ───

#[test]
fn idle_to_idle_same_value_write_is_noop_no_overlay_inserted() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    // Write Idle → Idle: Changed<DashState> fires but was_dashing=false AND
    // now_dashing=false → the system's enter/exit predicates are both false.
    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "Idle → Idle (same-value) must not change BoltLossBehavior"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "Idle → Idle must not insert OriginalBoltLossBehavior"
    );
}

// ── Behavior 6 (edge case) — no DashState mutation at all ───────────────────

#[test]
fn no_dash_state_mutation_leaves_idle_behavior_unchanged() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    // No mutation — Changed<DashState> is false; entity never enters query.
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(live, BoltLossBehavior::LifeLoss(1));
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none()
    );
}
