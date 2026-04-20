//! Group J — `reckless_dash_cleanup_node` `OnExit(NodeState::Playing)` cleanup
//! (Behaviors 60–62).
//!
//! Pins that `reckless_dash_cleanup_node`:
//! - Clears `RecklessDashDoubledBolts.0` on `OnExit(NodeState::Playing)`.
//! - Runs unconditionally — fires even when Reckless Dash is NOT in
//!   `ActiveProtocols` (NO `run_if(protocol_active)` gate).
//! - Does not resurrect stale state on node re-entry.
//!
//! State transitions use `app.update()` (not `tick(&mut app)`) because state
//! transitions fire on schedule edges — mirrors the Echo Strike
//! `cleanup_node.rs` convention.

use bevy::prelude::*;

use super::{
    super::system::RecklessDashDoubledBolts,
    helpers::{
        build_reckless_dash_app, seed_active_protocols_with_reckless_dash,
        spawn_bolt_with_base_damage, spawn_breaker_dashing, write_bolt_lost,
    },
};
use crate::prelude::*;

// ── Behavior 60 — cleanup clears RecklessDashDoubledBolts on OnExit ────────-

#[test]
fn exit_playing_clears_reckless_dash_doubled_bolts() {
    let mut app = build_reckless_dash_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    // Precondition: reckless_dash_double_penalty populated the tracking set.
    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .contains(&bolt),
        "precondition: RecklessDashDoubledBolts must contain the bolt after tick"
    );

    // Trigger OnExit(NodeState::Playing).
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .is_empty(),
        "RecklessDashDoubledBolts.0 must be empty after OnExit(Playing)"
    );
}

// ── Behavior 60 (edge case) — multiple entries all cleared ─────────────────-

#[test]
fn exit_playing_clears_multiple_doubled_bolts_entries() {
    let mut app = build_reckless_dash_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt_a, breaker);
    tick(&mut app);
    write_bolt_lost(&mut app, bolt_b, breaker);
    tick(&mut app);

    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .contains(&bolt_a)
            && app
                .world()
                .resource::<RecklessDashDoubledBolts>()
                .0
                .contains(&bolt_b),
        "precondition: both bolts populated in the tracking set"
    );

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .is_empty(),
        "cleanup must drain ALL entries, not just one"
    );
}

// ── Behavior 61 — cleanup runs even when Reckless Dash NOT active ──────────-

#[test]
fn cleanup_runs_even_when_reckless_dash_not_active() {
    let mut app = build_reckless_dash_app();
    // Do NOT seed ActiveProtocols — Reckless Dash is inactive.
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    // Manually populate the tracking set.
    app.world_mut()
        .resource_mut::<RecklessDashDoubledBolts>()
        .0
        .insert(bolt);

    // Precondition.
    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .contains(&bolt),
        "precondition: set contains the manually inserted bolt"
    );

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .is_empty(),
        "cleanup must fire on OnExit(Playing) even when Reckless Dash is inactive \
         — the cleanup system has NO run_if(protocol_active) gate"
    );
}

// ── Behavior 62 — re-entry does not resurrect stale state ──────────────────-

#[test]
fn re_entry_does_not_resurrect_stale_doubled_bolts_state() {
    let mut app = build_reckless_dash_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    // Exit Playing → cleanup fires.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .is_empty(),
        "precondition: set cleared on first exit"
    );

    // Re-enter Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    // No new writes — tick once and verify set remains empty.
    tick(&mut app);

    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .is_empty(),
        "set must remain empty after re-entry with no new writes"
    );
}

// ── Behavior 62 (edge case) — additional tick after re-entry still empty ───-

#[test]
fn additional_tick_after_re_entry_still_leaves_set_empty() {
    let mut app = build_reckless_dash_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    // Exit.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    // Re-enter.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    tick(&mut app);
    tick(&mut app); // one extra tick beyond Behavior 62's baseline

    assert!(
        app.world()
            .resource::<RecklessDashDoubledBolts>()
            .0
            .is_empty(),
        "deferred writes must not resurrect the set"
    );
}
