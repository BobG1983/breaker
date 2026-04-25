//! Direct unit coverage for `snapshot_pre_advance_state` and for the
//! "snapshot-populated" path in `apply_tier_regression`. Group C
//! integration tests in `register.rs` exercise both systems together
//! on `OnEnter(RunState::Node)`; this file isolates the snapshot system
//! and the snapshot-consumption path so regressions to either are
//! caught directly rather than observed through a composite assertion.

use bevy::prelude::{App, Update};

use super::{
    super::system::{TierRegressionPending, apply_tier_regression, snapshot_pre_advance_state},
    helpers::{install_config, install_outcome, install_pending, install_sequence_with_tiers, na},
};
use crate::{
    prelude::*,
    state::run::resources::{NodeOutcome, NodeSequence},
};

fn build_snapshot_app() -> App {
    TestAppBuilder::new()
        .with_system(Update, snapshot_pre_advance_state)
        .build()
}

fn build_apply_app() -> App {
    TestAppBuilder::new()
        .with_system(Update, apply_tier_regression)
        .build()
}

fn read_pending(app: &App) -> (Option<u32>, Option<u32>) {
    let p = app.world().resource::<TierRegressionPending>();
    (p.activation_tier, p.activation_node_index)
}

// ── snapshot_pre_advance_state — no-op when already captured ───────────────-

#[test]
fn snapshot_is_noop_when_activation_tier_already_some() {
    let mut app = build_snapshot_app();
    install_pending(&mut app);
    // Pre-populate the snapshot to simulate a prior OnEnter capture.
    app.world_mut()
        .resource_mut::<TierRegressionPending>()
        .activation_tier = Some(7);
    app.world_mut()
        .resource_mut::<TierRegressionPending>()
        .activation_node_index = Some(3);
    install_outcome(&mut app, 99, 42, 0);

    app.update();

    assert_eq!(
        read_pending(&app),
        (Some(7), Some(3)),
        "already-captured snapshot must be preserved verbatim — the live NodeOutcome must NOT overwrite it"
    );
}

// ── snapshot_pre_advance_state — no-op when NodeOutcome is absent ──────────-

#[test]
fn snapshot_is_noop_when_node_outcome_absent() {
    let mut app = build_snapshot_app();
    install_pending(&mut app);
    // Intentionally do NOT install NodeOutcome.

    app.update();

    assert_eq!(
        read_pending(&app),
        (None, None),
        "snapshot must leave fields as None when NodeOutcome is absent — apply falls back to live outcome"
    );
}

// ── snapshot_pre_advance_state — captures when fresh ───────────────────────-

#[test]
fn snapshot_captures_outcome_tier_and_node_index_when_fields_none() {
    let mut app = build_snapshot_app();
    install_pending(&mut app);
    install_outcome(&mut app, 4, 2, 1);

    app.update();

    assert_eq!(
        read_pending(&app),
        (Some(2), Some(4)),
        "fresh snapshot captures outcome.tier and outcome.node_index verbatim"
    );
}

// ── apply_tier_regression — uses snapshot values over live outcome ─────────-

/// Regression guard: when `TierRegressionPending.activation_tier` and
/// `.activation_node_index` are `Some`, `apply_tier_regression` must
/// prefer them over the live `NodeOutcome` values. This is the exact
/// contract the two-system architecture was designed to guarantee —
/// the snapshot captures pre-advance state and apply consumes it after
/// `advance_node` has potentially mutated the live outcome.
#[test]
fn apply_uses_snapshot_activation_fields_over_live_outcome() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    // Snapshot simulates pre-advance state: tier=2, node_index=3.
    app.world_mut()
        .resource_mut::<TierRegressionPending>()
        .activation_tier = Some(2);
    app.world_mut()
        .resource_mut::<TierRegressionPending>()
        .activation_node_index = Some(3);
    // Live outcome simulates post-advance state: tier=3 (Boss bumped),
    // node_index=4 (advance_node incremented).
    install_outcome(&mut app, 4, 3, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0), // 0
            na(NodeType::Boss, 0, 1.0),   // 1
            na(NodeType::Active, 1, 0.9), // 2
            na(NodeType::Boss, 2, 0.8),   // 3 ← snapshot-captured slot (pre-advance)
            na(NodeType::Active, 2, 0.8), // 4 ← live outcome.node_index (post-advance)
        ],
    );

    app.update();

    let outcome = app.world().resource::<NodeOutcome>();
    assert_eq!(
        outcome.tier, 1,
        "target_tier = snapshot.activation_tier(2) - tiers_back(1) = 1 — snapshot beats the live tier=3"
    );
    assert_eq!(
        outcome.position_in_tier, 0,
        "apply is the final authority — position_in_tier always 0"
    );

    let seq = &app.world().resource::<NodeSequence>().assignments;
    // splice_at = snapshot.activation_node_index(3) + 1 = 4.
    assert_eq!(
        seq[4],
        na(NodeType::Active, 1, 0.9),
        "tier-1 Active spliced at snapshot_node_index + 1 = 4 (not live+1 = 5)"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending removed after apply"
    );
}
