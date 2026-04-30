//! Regression — `reckless_dash_cleanup_node` on `OnExit(NodeState::Playing)`.
//!
//! Bug: when `NodeState::Playing` exits while the breaker is
//! `DashState::Dashing` with `double_penalty: true`, the
//! `reckless_dash_on_dash_transition` system does NOT run (it is gated on
//! `in_state(NodeState::Playing)`). As a result:
//! - `OriginalBoltLossBehavior` is orphaned on the breaker entity.
//! - `BoltLossBehavior` remains at the doubled value (`LifeLoss(2)` instead
//!   of `LifeLoss(1)`).
//! - `reset_breaker` runs on `OnEnter(NodeState::Loading)` but does not
//!   touch `BoltLossBehavior` or `OriginalBoltLossBehavior`.
//! - The next node begins with a permanently doubled penalty.
//!
//! The correct behavior: a dedicated `OnExit(NodeState::Playing)` cleanup
//! system restores `BoltLossBehavior` from `OriginalBoltLossBehavior` and
//! removes the overlay — unconditionally, regardless of whether the exit was
//! a normal dash completion or a mid-dash node transition.

use bevy::prelude::*;

use super::{
    super::system::{
        OriginalBoltLossBehavior, RecklessDashConfig, reckless_dash_cleanup_node,
        reckless_dash_on_dash_transition,
    },
    helpers::{
        force_dash_state, seed_active_protocols_with_reckless_dash, spawn_breaker_for_transition,
    },
};
use crate::{
    breaker::{
        components::{BoltLossBehavior, DashState},
        sets::BreakerSystems,
        systems::update_previous_dash_state,
    },
    mutators::protocols::{
        definition::ProtocolKind,
        resources::{ActiveProtocols, protocol_active},
    },
    prelude::*,
};

// ── App builder ─────────────────────────────────────────────────────────────

/// Test app that wires both `reckless_dash_on_dash_transition` (to set up the
/// doubled state) and `reckless_dash_cleanup_node` on
/// `OnExit(NodeState::Playing)` (the system under test).
///
/// Uses `wire()` to register all systems including the cleanup, then adds
/// `update_previous_dash_state` so the transition system can detect enter/exit.
fn build_reckless_dash_node_exit_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .build();
    app.world_mut().insert_resource(RecklessDashConfig {
        risky_zone_start:  0.7,
        damage_multiplier: 4.0,
        double_penalty:    true,
    });

    app.configure_sets(
        FixedUpdate,
        BreakerSystems::UpdateState.before(BreakerSystems::UpdatePreviousState),
    );
    app.add_systems(
        FixedUpdate,
        (
            update_previous_dash_state
                .in_set(BreakerSystems::UpdatePreviousState)
                .after(BreakerSystems::UpdateState),
            reckless_dash_on_dash_transition
                .after(BreakerSystems::UpdateState)
                .before(BreakerSystems::UpdatePreviousState)
                .run_if(protocol_active(ProtocolKind::RecklessDash)),
        ),
    );
    app.add_systems(OnExit(NodeState::Playing), reckless_dash_cleanup_node);
    app
}

// ── Behavior: node exit while dashing restores BoltLossBehavior ─────────────
//
// Regression: without a cleanup system on OnExit(NodeState::Playing),
// BoltLossBehavior stays doubled (LifeLoss(2)) and OriginalBoltLossBehavior
// is orphaned when the node exits while the breaker is mid-dash.

#[test]
fn node_exit_while_dashing_restores_bolt_loss_behavior_to_original() {
    let mut app = build_reckless_dash_node_exit_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    // Spawn breaker in Idle with BoltLossBehavior::LifeLoss(1).
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    // Tick 1: Idle → Dashing transition. reckless_dash_on_dash_transition
    // inserts OriginalBoltLossBehavior(LifeLoss(1)) and doubles live
    // BoltLossBehavior to LifeLoss(2).
    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    // Verify the doubling happened (precondition for the regression).
    let live_after_enter = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior after dash enter");
    assert_eq!(
        live_after_enter,
        BoltLossBehavior::LifeLoss(2),
        "precondition: BoltLossBehavior must be doubled to LifeLoss(2) after dash enter, \
         got {live_after_enter:?}"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_some(),
        "precondition: OriginalBoltLossBehavior must be present after dash enter"
    );

    // Exit NodeState::Playing while the breaker is STILL dashing.
    // This triggers OnExit(NodeState::Playing) → reckless_dash_cleanup_node.
    // reckless_dash_on_dash_transition does NOT run (gated on Playing).
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    // Assert: BoltLossBehavior must be restored to the original LifeLoss(1).
    let live_after_exit = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must still have BoltLossBehavior after node exit");
    assert_eq!(
        live_after_exit,
        BoltLossBehavior::LifeLoss(1),
        "BoltLossBehavior must be restored to LifeLoss(1) after OnExit(NodeState::Playing) \
         cleanup — was doubled to LifeLoss(2) during dash and must not persist to next node, \
         got {live_after_exit:?}"
    );

    // Assert: OriginalBoltLossBehavior overlay must be removed.
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "OriginalBoltLossBehavior must be absent after OnExit(NodeState::Playing) cleanup — \
         overlay was orphaned during mid-dash node exit and must be cleaned up"
    );
}

// ── Edge case: node exit while NOT dashing (Idle) leaves behavior unchanged ──
//
// Cleanup must restore the dashing breaker while leaving the idle breaker
// untouched. The two-entity setup gives the test something that will fail
// against the no-op stub: the dashing breaker stays at LifeLoss(2) instead of
// being restored to LifeLoss(1), proving the system must do real work.

#[test]
fn node_exit_while_idle_leaves_bolt_loss_behavior_unchanged() {
    let mut app = build_reckless_dash_node_exit_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    // Breaker A — stays Idle, no overlay. Cleanup must be a no-op for it.
    let idle_breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    // Breaker B — mid-dash with doubled BoltLossBehavior and overlay present.
    // Simulate the post-transition state that reckless_dash_on_dash_transition
    // would have produced: OriginalBoltLossBehavior(LifeLoss(1)) inserted and
    // live BoltLossBehavior doubled to LifeLoss(2).
    let dashing_breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(2),
        DashState::Dashing,
        DashState::Dashing,
    );
    app.world_mut()
        .entity_mut(dashing_breaker)
        .insert(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));

    // Exit NodeState::Playing while one breaker is dashing and one is idle.
    // This triggers OnExit(NodeState::Playing) → reckless_dash_cleanup_node.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    // Assert: dashing breaker must be restored to LifeLoss(1).
    // FAILS at RED because the stub does nothing — dashing breaker stays at
    // LifeLoss(2) — and passes only when the cleanup system restores it.
    let dashing_live = *app
        .world()
        .get::<BoltLossBehavior>(dashing_breaker)
        .expect("dashing breaker must still have BoltLossBehavior after node exit");
    assert_eq!(
        dashing_live,
        BoltLossBehavior::LifeLoss(1),
        "dashing breaker BoltLossBehavior must be restored to LifeLoss(1) by cleanup — \
         was LifeLoss(2) during dash, got {dashing_live:?}"
    );

    // Assert: dashing breaker overlay must be removed.
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(dashing_breaker)
            .is_none(),
        "OriginalBoltLossBehavior must be absent from the dashing breaker after cleanup"
    );

    // Assert: idle breaker BoltLossBehavior must remain LifeLoss(1) unchanged.
    let idle_live = *app
        .world()
        .get::<BoltLossBehavior>(idle_breaker)
        .expect("idle breaker must still have BoltLossBehavior after node exit");
    assert_eq!(
        idle_live,
        BoltLossBehavior::LifeLoss(1),
        "cleanup must be a no-op when breaker is not mid-dash — BoltLossBehavior must \
         remain LifeLoss(1), got {idle_live:?}"
    );

    // Assert: idle breaker must never have had an overlay inserted.
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(idle_breaker)
            .is_none(),
        "no overlay must appear on an idle breaker after node exit"
    );
}
