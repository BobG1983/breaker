//! Group A — `reckless_dash_on_dash_transition` component mutation on dash
//! enter / exit (Behaviors 1–11).
//!
//! Pins the new mutation-on-transition contract:
//! - Idle → Dashing: saves `BoltLossBehavior` in `OriginalBoltLossBehavior`
//!   overlay and doubles the live component.
//! - Dashing → {anything else}: restores from overlay, removes overlay.
//! - Mid-dash ticks (no `Changed<DashState>`): no-op.
//! - `double_penalty: false` or absent config: short-circuit, no mutation.
//! - `protocol_active` gate: system skipped when Reckless Dash inactive.
//! - Multi-breaker isolation: only the transitioning breaker is mutated.

use bevy::prelude::*;

use super::{
    super::system::{OriginalBoltLossBehavior, RecklessDashConfig},
    helpers::{
        build_reckless_dash_transition_app, build_reckless_dash_transition_app_no_config,
        force_dash_state, install_reckless_dash_config, original_behavior,
        seed_active_protocols_with_reckless_dash, spawn_breaker_for_transition,
    },
};
use crate::{
    breaker::components::{BoltLossBehavior, DashState},
    prelude::*,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}

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

    assert_eq!(
        original_behavior(&app, breaker),
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

    assert_eq!(
        original_behavior(&app, breaker),
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

// ── Behavior 7 — double_penalty: false short-circuits mutation on dash enter ─

#[test]
fn double_penalty_false_suppresses_overlay_and_doubling_on_dash_enter() {
    let mut app = build_reckless_dash_transition_app();
    install_reckless_dash_config(
        &mut app,
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    false,
        },
    );
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

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
        "double_penalty:false must NOT double BoltLossBehavior"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "double_penalty:false must NOT insert OriginalBoltLossBehavior"
    );
}

// ── Behavior 7 (edge case) — TimeLoss(5.0) also suppressed ─────────────────

#[test]
fn double_penalty_false_suppresses_timeloss_doubling_on_dash_enter() {
    let mut app = build_reckless_dash_transition_app();
    install_reckless_dash_config(
        &mut app,
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    false,
        },
    );
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

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
        (actual - 5.0).abs() < f32::EPSILON,
        "double_penalty:false must leave TimeLoss(5.0) unchanged, got {actual}"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "double_penalty:false must NOT insert OriginalBoltLossBehavior"
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

// ── Behavior 9 — RecklessDashConfig absent → safe no-op ────────────────────

#[test]
fn absent_config_on_dash_enter_is_safe_noop_no_overlay_inserted() {
    let mut app = build_reckless_dash_transition_app_no_config();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
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
        BoltLossBehavior::LifeLoss(1),
        "absent config must not mutate BoltLossBehavior"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "absent config must not insert OriginalBoltLossBehavior"
    );
}

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
