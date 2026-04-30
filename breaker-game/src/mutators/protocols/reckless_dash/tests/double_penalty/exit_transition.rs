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

// ── Behavior 3 — Dashing → Idle restores BoltLossBehavior from the overlay ─

#[test]
fn dashing_to_idle_restores_lifeloss_one_from_overlay() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    // Spawn breaker already mid-dash with overlay in place (as if a prior tick
    // performed the Idle → Dashing transition).
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(2),
        DashState::Dashing,
        DashState::Dashing,
    );
    app.world_mut()
        .entity_mut(breaker)
        .insert(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));

    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "Dashing → Idle must restore BoltLossBehavior to LifeLoss(1)"
    );
}

// ── Behavior 3 (edge case) — TimeLoss(10.0) restored to TimeLoss(5.0) ───────

#[test]
fn dashing_to_idle_restores_timeloss_five_from_overlay() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::TimeLoss(10.0),
        DashState::Dashing,
        DashState::Dashing,
    );
    app.world_mut()
        .entity_mut(breaker)
        .insert(OriginalBoltLossBehavior(BoltLossBehavior::TimeLoss(5.0)));

    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    let BoltLossBehavior::TimeLoss(actual) = live else {
        panic!("expected TimeLoss variant, got {live:?}");
    };
    assert!(
        (actual - 5.0).abs() < f32::EPSILON,
        "Dashing → Idle must restore TimeLoss(5.0) from overlay, got TimeLoss({actual})"
    );
}

// ── Behavior 3 (edge case) — Dashing → Braking also restores ────────────────

#[test]
fn dashing_to_braking_restores_lifeloss_one_from_overlay() {
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

    force_dash_state(&mut app, breaker, DashState::Braking);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "Dashing → Braking must restore BoltLossBehavior to LifeLoss(1)"
    );
}

// ── Behavior 3 (edge case) — Dashing → Settling also restores ───────────────

#[test]
fn dashing_to_settling_restores_lifeloss_one_from_overlay() {
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

    force_dash_state(&mut app, breaker, DashState::Settling);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "Dashing → Settling must restore BoltLossBehavior to LifeLoss(1)"
    );
}

// ── Behavior 4 — Dashing → Idle removes the overlay component ───────────────

#[test]
fn dashing_to_idle_removes_overlay_component() {
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

    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "Dashing → Idle must remove OriginalBoltLossBehavior component"
    );
}

// ── Behavior 4 (edge case) — Dashing → Braking removes overlay ──────────────

#[test]
fn dashing_to_braking_removes_overlay_component() {
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

    force_dash_state(&mut app, breaker, DashState::Braking);
    tick(&mut app);

    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "Dashing → Braking must remove OriginalBoltLossBehavior component"
    );
}

// ── Behavior 4 (edge case) — Dashing → Settling removes overlay ─────────────

#[test]
fn dashing_to_settling_removes_overlay_component() {
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

    force_dash_state(&mut app, breaker, DashState::Settling);
    tick(&mut app);

    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "Dashing → Settling must remove OriginalBoltLossBehavior component"
    );
}

// ── Behavior 8 — dash exit without overlay is a safe no-op ─────────────────

#[test]
fn dashing_to_idle_without_overlay_is_safe_noop() {
    let mut app = build_reckless_dash_transition_app();
    seed_canonical(&mut app);

    // Breaker mid-dash but NO overlay (e.g., protocol activated mid-dash).
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Dashing,
        DashState::Dashing,
    );
    // No OriginalBoltLossBehavior inserted.

    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app); // must not panic

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "dash exit without overlay must not corrupt BoltLossBehavior"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "dash exit without overlay must NOT spuriously insert OriginalBoltLossBehavior"
    );
}
