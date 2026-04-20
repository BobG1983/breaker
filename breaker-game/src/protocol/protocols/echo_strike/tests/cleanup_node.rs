//! Group F — `echo_strike_cleanup_node` (Behaviors 42–45).
//!
//! Pins the `OnExit(NodeState::Playing)` cleanup:
//! - Removes ALL `EchoNetwork` and `EchoPrimed` from every bolt.
//! - Runs unconditionally — even when Echo Strike is NOT active.
//! - Tolerates absent `EchoStrikeConfig`.
//! - Echoes do NOT persist across a node (re-entry does not resurrect them).

use bevy::prelude::*;

use super::{
    super::system::{EchoNetwork, EchoPrimed},
    helpers::{
        build_echo_strike_app, build_echo_strike_app_no_config, install_echo_primed,
        seed_active_protocols_with_echo_strike, spawn_bolt_with_base_damage,
        spawn_bolt_with_echo_network, spawn_cell_empty,
    },
};
use crate::prelude::*;

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_echo_strike(app, 3, 0.5, 0.25, 0.1);
}

// ── Behavior 42 — OnExit removes EchoNetwork + EchoPrimed on all bolts ─────-

#[test]
fn exit_playing_removes_all_echo_network_and_echo_primed() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);

    let cell_a = spawn_cell_empty(&mut app);
    let cell_b = spawn_cell_empty(&mut app);
    let cell_c = spawn_cell_empty(&mut app);
    let cell_d = spawn_cell_empty(&mut app);

    // bolt_full has both components.
    let bolt_full = spawn_bolt_with_echo_network(&mut app, 10.0, vec![cell_a, cell_b, cell_c]);
    install_echo_primed(&mut app, bolt_full);
    // bolt_net has only the network.
    let bolt_net = spawn_bolt_with_echo_network(&mut app, 10.0, vec![cell_d]);
    // bolt_marker has only the primed marker (no network).
    let bolt_marker = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_echo_primed(&mut app, bolt_marker);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    for bolt in [bolt_full, bolt_net, bolt_marker] {
        assert!(
            app.world().get::<EchoNetwork>(bolt).is_none(),
            "entity {bolt:?} should have no EchoNetwork after cleanup"
        );
        assert!(
            app.world().get::<EchoPrimed>(bolt).is_none(),
            "entity {bolt:?} should have no EchoPrimed after cleanup"
        );
    }
}

// ── Behavior 42 edge (42a) — idempotent across multiple transitions ────────-

#[test]
fn cleanup_node_is_idempotent_across_transitions() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);
    install_echo_primed(&mut app, bolt);

    // First exit.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    assert!(app.world().get::<EchoNetwork>(bolt).is_none());
    assert!(app.world().get::<EchoPrimed>(bolt).is_none());

    // Re-enter and re-exit.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update(); // must not panic

    assert!(app.world().get::<EchoNetwork>(bolt).is_none());
    assert!(app.world().get::<EchoPrimed>(bolt).is_none());
}

// ── Behavior 43 — cleanup runs even when Echo Strike NOT active ────────────-

#[test]
fn cleanup_runs_even_when_echo_strike_not_active() {
    let mut app = build_echo_strike_app();
    // Do NOT seed ActiveProtocols.
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);
    install_echo_primed(&mut app, bolt);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(
        app.world().get::<EchoNetwork>(bolt).is_none(),
        "cleanup must fire even without Echo Strike active"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "cleanup must fire even without Echo Strike active"
    );
}

// ── Behavior 44 — cleanup tolerates absent config ──────────────────────────-

#[test]
fn cleanup_tolerates_absent_config() {
    let mut app = build_echo_strike_app_no_config();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let b = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a, b]);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update(); // must not panic

    assert!(
        app.world().get::<EchoNetwork>(bolt).is_none(),
        "cleanup removes EchoNetwork even without config"
    );
}

// ── Behavior 45 — echoes do NOT persist across node re-entry ───────────────-

#[test]
fn echoes_do_not_persist_across_node_re_entry() {
    let mut app = build_echo_strike_app();
    seed_canonical(&mut app);
    let a = spawn_cell_empty(&mut app);
    let bolt = spawn_bolt_with_echo_network(&mut app, 10.0, vec![a]);
    install_echo_primed(&mut app, bolt);

    // Exit Playing → cleanup fires.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    assert!(app.world().get::<EchoNetwork>(bolt).is_none());
    assert!(app.world().get::<EchoPrimed>(bolt).is_none());

    // Re-enter Playing and tick — must NOT resurrect echoes.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    tick(&mut app);

    assert!(
        app.world().get::<EchoNetwork>(bolt).is_none(),
        "EchoNetwork must NOT be re-attached on node re-entry"
    );
    assert!(
        app.world().get::<EchoPrimed>(bolt).is_none(),
        "EchoPrimed must NOT be re-attached on node re-entry"
    );
}
