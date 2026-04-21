//! Group C — `register` wiring + run-condition gating (Behaviors 21–23, 23a).
//!
//! Pins that `register` wires `apply_tier_regression` on
//! `OnEnter(RunState::Node)` with `.after(NodeSystems::AdvanceNode)` and both
//! run-conditions (`protocol_active(ProtocolKind::TierRegression)` +
//! `resource_exists::<TierRegressionPending>`), and verifies the ordering
//! witness by observing `NodeOutcome.position_in_tier`. `apply_tier_regression`
//! is the final authority on `outcome.tier` and `outcome.position_in_tier`;
//! the Boss-boundary bug (`advance_node`'s tier-increment cancelling apply's
//! rewind) is guarded against by running apply last.

use bevy::prelude::App;

use super::{
    super::system::{TierRegressionConfig, TierRegressionPending},
    helpers::{
        build_register_app, enter_run_state_node, install_config, install_outcome, install_pending,
        install_sequence_with_tiers, leave_run_state_node_to, na,
        seed_active_protocols_with_tier_regression,
    },
};
use crate::{
    prelude::*,
    state::run::resources::{NodeOutcome, NodeSequence},
};

fn read_outcome(app: &App) -> (u32, u32, u32) {
    let o = app.world().resource::<NodeOutcome>();
    (o.node_index, o.tier, o.position_in_tier)
}

fn sequence_len(app: &App) -> usize {
    app.world().resource::<NodeSequence>().assignments.len()
}

// ── 21 — full integration: splice + advance_node on OnEnter(RunState::Node) ─

#[test]
fn register_wires_apply_before_advance_node_and_splices_on_enter_run_state_node() {
    let mut app = build_register_app();
    seed_active_protocols_with_tier_regression(&mut app, 1);
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 4, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0), // 0
            na(NodeType::Boss, 0, 1.0),   // 1
            na(NodeType::Active, 1, 0.9), // 2
            na(NodeType::Boss, 1, 0.9),   // 3
            na(NodeType::Active, 2, 0.8), // 4 ← current (Active → advance_node will NOT bump tier)
            na(NodeType::Boss, 2, 0.8),   // 5
        ],
    );

    enter_run_state_node(&mut app);

    assert_eq!(
        sequence_len(&app),
        8,
        "splice must occur — length grows from 6 to 8"
    );
    let seq = &app.world().resource::<NodeSequence>().assignments;
    assert_eq!(
        seq[5],
        na(NodeType::Active, 1, 0.9),
        "first spliced tier-1 node at index 5"
    );
    assert_eq!(
        seq[6],
        na(NodeType::Boss, 1, 0.9),
        "second spliced tier-1 node at index 6"
    );

    let (node_index, tier, position) = read_outcome(&app);
    assert_eq!(
        tier, 1,
        "apply_tier_regression rewound tier 2 → 1; advance_node (Active completed) did not touch"
    );
    assert_eq!(
        position, 0,
        "apply runs AFTER advance_node and is the final authority — \
         position is always 0 after tier regression"
    );
    assert_eq!(node_index, 5, "advance_node incremented node_index 4 → 5");

    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending removed after splice"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 21 edge — one-shot: exit and re-enter RunState::Node is a no-op ────────-

#[test]
fn re_entering_run_state_node_after_splice_is_a_no_op() {
    let mut app = build_register_app();
    seed_active_protocols_with_tier_regression(&mut app, 1);
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 4, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
            na(NodeType::Active, 2, 0.8), // 4 ← current
            na(NodeType::Boss, 2, 0.8),
        ],
    );

    enter_run_state_node(&mut app);
    assert_eq!(sequence_len(&app), 8, "first entry splices");

    leave_run_state_node_to(&mut app, RunState::ChipSelect);
    enter_run_state_node(&mut app);

    assert_eq!(
        sequence_len(&app),
        8,
        "second OnEnter(RunState::Node) must be a no-op for apply_tier_regression"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending still absent after re-entry"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 22 — run-condition gate off when TierRegression not in ActiveProtocols ─-

#[test]
fn apply_does_not_run_when_tier_regression_not_in_active_protocols() {
    let mut app = build_register_app();
    // Do NOT seed ActiveProtocols with TierRegression.
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 4, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
            na(NodeType::Active, 2, 0.8), // 4 ← current
            na(NodeType::Boss, 2, 0.8),
        ],
    );

    enter_run_state_node(&mut app);

    assert_eq!(
        sequence_len(&app),
        6,
        "sequence length unchanged — apply_tier_regression gated off"
    );
    let (node_index, tier, position) = read_outcome(&app);
    assert_eq!(tier, 2, "tier NOT rewound — gate held apply off");
    assert_eq!(position, 1, "advance_node incremented position from 0 → 1");
    assert_eq!(node_index, 5, "advance_node incremented 4 → 5");
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_some(),
        "pending still present — apply was gated off and never removed it"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 22 edge — late-seed ActiveProtocols + re-entry fires splice ───────────-

#[test]
fn gate_is_per_entry_not_latched_active_protocols() {
    let mut app = build_register_app();
    // Do NOT seed ActiveProtocols with TierRegression yet.
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 3, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9), // 3 ← current
            na(NodeType::Active, 2, 0.8),
            na(NodeType::Boss, 2, 0.8),
        ],
    );

    // First entry: gate held — apply_tier_regression does not run.
    enter_run_state_node(&mut app);
    assert_eq!(
        sequence_len(&app),
        6,
        "first entry: sequence unchanged — gate held apply off"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_some(),
        "first entry: pending still present — apply was gated off"
    );

    // Flip the gate on: seed ActiveProtocols with TierRegression.
    seed_active_protocols_with_tier_regression(&mut app, 1);

    // Leave RunState::Node and re-enter to trigger OnEnter again.
    leave_run_state_node_to(&mut app, RunState::ChipSelect);
    enter_run_state_node(&mut app);

    assert_eq!(
        sequence_len(&app),
        8,
        "re-entry: splice fires — run-condition gate is re-evaluated per OnEnter, not latched"
    );
    let (_node_index, _tier, position) = read_outcome(&app);
    assert_eq!(
        position, 0,
        "apply runs AFTER advance_node and is the final authority — \
         position is always 0 after tier regression"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "re-entry: pending removed after splice"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 23 — run-condition gate off when TierRegressionPending is absent ───────-

#[test]
fn apply_does_not_run_when_pending_resource_absent() {
    let mut app = build_register_app();
    seed_active_protocols_with_tier_regression(&mut app, 1);
    install_config(&mut app, 1);
    // No install_pending().
    install_outcome(&mut app, 4, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
            na(NodeType::Active, 2, 0.8), // 4 ← current
            na(NodeType::Boss, 2, 0.8),
        ],
    );

    enter_run_state_node(&mut app);

    assert_eq!(
        sequence_len(&app),
        6,
        "sequence length unchanged — apply_tier_regression gated off by missing pending"
    );
    let (node_index, tier, position) = read_outcome(&app);
    assert_eq!(tier, 2, "tier NOT rewound");
    assert_eq!(position, 1, "advance_node incremented position → 1");
    assert_eq!(node_index, 5);
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending still absent — system was gated off by missing pending resource"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 23 edge — late-insert pending + re-entry fires splice ─────────────────-

#[test]
fn gate_is_per_entry_not_latched_pending() {
    let mut app = build_register_app();
    seed_active_protocols_with_tier_regression(&mut app, 1);
    install_config(&mut app, 1);
    // Do NOT install_pending() yet.
    install_outcome(&mut app, 4, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
            na(NodeType::Active, 2, 0.8), // 4 ← current
            na(NodeType::Boss, 2, 0.8),
        ],
    );

    // First entry: gate held by missing pending — apply_tier_regression does not run.
    enter_run_state_node(&mut app);
    assert_eq!(
        sequence_len(&app),
        6,
        "first entry: sequence unchanged — gate held apply off"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "first entry: pending still absent — apply was gated off"
    );

    // Flip the gate on: install pending.
    install_pending(&mut app);

    // Leave RunState::Node and re-enter to trigger OnEnter again.
    leave_run_state_node_to(&mut app, RunState::ChipSelect);
    enter_run_state_node(&mut app);

    assert_eq!(
        sequence_len(&app),
        8,
        "re-entry: splice fires — run-condition gate is re-evaluated per OnEnter, not latched"
    );

    // Splice position + outcome — trace of the second entry:
    //   snapshot captures pre-advance outcome = (node_index=5, tier=2).
    //   advance_node: sequence[5] = Boss tier 2 → tier += 1 → 3;
    //     position = 0; node_index = 6.
    //   apply: target_tier = snapshot.tier(2) - tiers_back(1) = 1;
    //     splice_at = snapshot.node_index(5) + 1 = 6 (end of original);
    //     tier = 1 (overwrites post-advance 3); position = 0.
    let seq = &app.world().resource::<NodeSequence>().assignments;
    assert_eq!(
        seq[6],
        na(NodeType::Active, 1, 0.9),
        "re-entry: tier-1 Active spliced at snapshot_node_index + 1 = 6"
    );
    assert_eq!(
        seq[7],
        na(NodeType::Boss, 1, 0.9),
        "re-entry: tier-1 Boss spliced at index 7"
    );
    let (node_index, tier, position) = read_outcome(&app);
    assert_eq!(
        tier, 1,
        "re-entry: apply rewound tier from snapshot(2) to target_tier(1); advance_node's Boss-bump to 3 does not leak through"
    );
    assert_eq!(
        position, 0,
        "re-entry: apply is the final authority — position_in_tier is 0 after the splice"
    );
    assert_eq!(
        node_index, 6,
        "re-entry: advance_node incremented node_index 5 → 6 (apply never touches node_index)"
    );

    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "re-entry: pending removed after splice"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 23a — ordering witness: apply runs AFTER advance_node on OnEnter ───────-

#[test]
fn apply_tier_regression_runs_after_advance_node_on_on_enter_run_state_node() {
    let mut app = build_register_app();
    seed_active_protocols_with_tier_regression(&mut app, 1);
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 0, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 1, 0.9), // length 1; no tier-0 entries
        ],
    );

    enter_run_state_node(&mut app);

    let (node_index, tier, position) = read_outcome(&app);
    assert_eq!(node_index, 1, "advance_node incremented node_index 0 → 1");
    assert_eq!(
        tier, 0,
        "apply_tier_regression rewound tier 1 → 0; advance_node did not touch tier (Active completed)"
    );
    assert_eq!(
        position, 0,
        "ORDERING WITNESS: apply runs AFTER advance_node and is the final authority — \
         position is always 0 after tier regression \
         (advance_node would have incremented position → 1, but apply then reset it → 0)"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending removed after splice attempt (empty splice still clears pending)"
    );
    assert_eq!(
        sequence_len(&app),
        1,
        "no tier-0 source nodes — splice was empty but rewind still occurred"
    );
}

// ── 23a — Boss-boundary regression: apply must win over advance_node ───────-

/// Boss-boundary regression guard.
///
/// Before the fix, `apply_tier_regression` ran `.before(NodeSystems::AdvanceNode)`:
/// apply rewound `outcome.tier` to `target_tier`, then `advance_node` saw the
/// Boss at `sequence[node_index]` and did `outcome.tier += 1`. The net effect
/// was that the tier rewind was **silently cancelled** whenever the just-
/// completed node was a Boss — a real gameplay bug.
///
/// The fix makes `apply_tier_regression` the final authority over
/// `outcome.tier` and `outcome.position_in_tier` by running
/// `.after(NodeSystems::AdvanceNode)`. After both systems run, `outcome.tier
/// == target_tier` regardless of whether the completed node was a Boss.
#[test]
fn boss_boundary_activation_rewinds_tier_and_cancels_advance_node_increment() {
    let mut app = build_register_app();
    seed_active_protocols_with_tier_regression(&mut app, 1);
    install_config(&mut app, 1);
    install_pending(&mut app);
    // node_index=3, tier=2, position=0 → we are about to complete the Boss at
    // sequence[3] (tier 2). Under the buggy ordering this Boss-increment from
    // advance_node silently cancels apply's rewind.
    install_outcome(&mut app, 3, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0), // 0 — tier 0
            na(NodeType::Boss, 0, 1.0),   // 1 — tier 0 Boss
            na(NodeType::Active, 1, 0.9), // 2 — tier 1 Active
            na(NodeType::Boss, 2, 0.8),   // 3 ← current (tier 2 Boss just completed)
        ],
    );

    enter_run_state_node(&mut app);

    let (node_index, tier, position) = read_outcome(&app);
    assert_eq!(
        tier, 1,
        "Boss-boundary regression: advance_node must NOT carry tier back up after apply rewinds it — \
         apply is the final authority and target_tier = 2 - tiers_back(1) = 1"
    );
    assert_eq!(
        position, 0,
        "apply runs AFTER advance_node and is the final authority — \
         position is always 0 after tier regression"
    );
    assert_eq!(
        node_index, 4,
        "advance_node incremented node_index 3 → 4 (apply never touches node_index)"
    );

    // Splice occurred at node_index + 1 == 4 with the tier-1 assignments
    // (one Active at original index 2).
    assert_eq!(
        sequence_len(&app),
        5,
        "splice must occur — length grows from 4 to 5 (one tier-1 Active spliced in)"
    );
    let seq = &app.world().resource::<NodeSequence>().assignments;
    assert_eq!(
        seq[4],
        na(NodeType::Active, 1, 0.9),
        "spliced tier-1 node at index 4 (node_index + 1)"
    );

    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending removed after splice"
    );
}
