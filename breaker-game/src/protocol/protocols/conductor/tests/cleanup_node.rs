//! Group C — no per-node state to clean (Behaviors 25–26).
//!
//! Pins that `ConductorConfig` survives `OnExit(NodeState::Playing)` (it is
//! per-run, not per-node), that bolts retain their post-swap state across the
//! transition, and that re-entering `NodeState::Playing` restores the swap
//! behaviour without any hidden leftover state.

use bevy::prelude::*;

use super::{
    super::system::ConductorConfig,
    helpers::{
        bound_fingerprints, build_conductor_app, has_extra, has_primary, make_distinct_bound,
        seed_active_protocols_with_conductor, spawn_dummy_breaker, spawn_extra_bolt_with_bound,
        spawn_primary_bolt_with_bound, write_bump_performed,
    },
};
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── Behavior 25 — no per-node state to clean: swap survives OnExit ──────────

#[test]
fn on_exit_node_playing_does_not_disturb_post_swap_state() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    // Trigger a swap.
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    // Precondition — swap happened.
    assert!(has_primary(&app, extra), "precondition: extra promoted");
    assert!(has_extra(&app, primary), "precondition: primary demoted");

    // Drive OnExit(NodeState::Playing).
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    // ConductorConfig survives — it's per-run, not per-node.
    assert!(
        app.world().get_resource::<ConductorConfig>().is_some(),
        "ConductorConfig must survive OnExit(NodeState::Playing)"
    );

    // Bolts still exist with their post-swap state.
    assert!(has_primary(&app, extra), "post-OnExit: extra still primary");
    assert!(has_extra(&app, primary), "post-OnExit: primary still extra");
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PRIMARY_BOUND".to_string()],
        "post-OnExit: bumped bolt's bound unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["EXTRA_BOUND".to_string()],
        "post-OnExit: old primary's bound unchanged"
    );
}

// ── Behavior 25 (edge case) — re-entering Playing reopens the gate ──────────

#[test]
fn re_entering_node_playing_reopens_gate_without_leaked_state() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    // First swap.
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    // Exit Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get_resource::<ConductorConfig>().is_some(),
        "ConductorConfig survives OnExit"
    );

    // Re-enter Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    // Now `extra` is primary and `primary` is extra. A Perfect bump on
    // `primary` (the current extra) must swap them back.
    write_bump_performed(&mut app, breaker, Some(primary), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "re-entry: gate reopened — swap ran, primary promoted back"
    );
    assert!(has_extra(&app, extra), "re-entry: extra demoted back");
}

// ── Behavior 26 — OnExit without prior swap leaves state untouched ──────────

#[test]
fn on_exit_node_playing_without_prior_swap_leaves_bolts_and_config_unchanged() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get_resource::<ConductorConfig>().is_some(),
        "ConductorConfig unchanged"
    );
    assert!(has_primary(&app, primary), "primary unchanged");
    assert!(has_extra(&app, extra), "extra unchanged");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()]
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()]
    );
}
