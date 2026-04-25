//! Groups B and B′ — `apply_tier_regression` core splice behavior and
//! one-shot / no-op guards (Behaviors 6–20).
//!
//! All tests wire `apply_tier_regression` on `Update` via
//! `build_apply_app()` (no state hierarchy, no run-if gate). Group C covers
//! schedule wiring and ordering.

use bevy::prelude::App;

use super::{
    super::system::{TierRegressionConfig, TierRegressionPending},
    helpers::{
        build_apply_app, install_config, install_outcome, install_outcome_full, install_pending,
        install_sequence_with_tiers, na,
    },
};
use crate::{
    prelude::*,
    state::run::resources::{NodeOutcome, NodeResult, NodeSequence},
};

fn read_sequence(app: &App) -> Vec<(NodeType, u32, f32)> {
    app.world()
        .resource::<NodeSequence>()
        .assignments
        .iter()
        .map(|a| (a.node_type, a.tier_index, a.timer_mult))
        .collect()
}

fn read_outcome(app: &App) -> (u32, u32, u32) {
    let o = app.world().resource::<NodeOutcome>();
    (o.node_index, o.tier, o.position_in_tier)
}

// ── 6 — tier-1 → tier-0 splice with `tiers_back: 1` at boss clear ──────────-

#[test]
fn splice_tier_0_nodes_into_position_after_current_when_tiers_back_is_one() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    app.update();

    let seq = read_sequence(&app);
    assert_eq!(
        seq,
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
        ],
        "tier-0 nodes must be spliced at index 3 (after the current node at index 2)"
    );
    assert_eq!(read_outcome(&app), (2, 0, 0));
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending must be removed after splice"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config must be preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 6 edge — spliced clones independent from originals (value equality) ────-

#[test]
fn spliced_clones_equal_originals_by_value() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    app.update();

    let assignments = &app.world().resource::<NodeSequence>().assignments;
    // Pre-existing tier-0 nodes at indices 0..=2 unchanged.
    assert_eq!(assignments[0], na(NodeType::Active, 0, 1.0));
    assert_eq!(assignments[1], na(NodeType::Active, 0, 1.0));
    assert_eq!(assignments[2], na(NodeType::Boss, 0, 1.0));
    // Spliced clones at indices 3..=5 — equal by value to the originals.
    assert_eq!(assignments[3], na(NodeType::Active, 0, 1.0));
    assert_eq!(assignments[4], na(NodeType::Active, 0, 1.0));
    assert_eq!(assignments[5], na(NodeType::Boss, 0, 1.0));
}

// ── 7 — splice position is `node_index + 1`, not end of sequence ───────────-

#[test]
fn splice_lands_at_node_index_plus_one_not_end_of_sequence() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 3, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0), // 0
            na(NodeType::Boss, 0, 1.0),   // 1
            na(NodeType::Active, 1, 0.9), // 2
            na(NodeType::Boss, 1, 0.9),   // 3 ← current
            na(NodeType::Active, 2, 0.8), // 4
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 2, 0.8),
        ],
        "spliced nodes land at indices 4–5 (after current at index 3), before index 4's tail"
    );
    assert_eq!(read_outcome(&app), (3, 0, 0));
}

// ── 7 edge — node_index 0 with single-tier sequence splices at index 1 ─────-

#[test]
fn splice_with_node_index_zero_lands_at_index_one() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 0, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 1, 0.9), // 0 ← current
            na(NodeType::Active, 0, 1.0), // 1 (pre-existing tier-0 source to splice)
            na(NodeType::Boss, 0, 1.0),   // 2
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 1, 0.9),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
        ],
        "spliced tier-0 nodes land at index 1 (right after current node at 0)"
    );
    assert_eq!(read_outcome(&app), (0, 0, 0));
}

// ── 8 — splice clamped to end when node_index + 1 > len ────────────────────-

#[test]
fn splice_position_clamped_to_end_when_node_index_past_sequence() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 4, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
        ],
        "spliced tier-0 nodes must be appended at the end when node_index + 1 > len"
    );
    assert_eq!(read_outcome(&app), (4, 0, 0));
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
    );
}

// ── 8 edge — node_index + 1 == len exactly (splice appends) ────────────────-

#[test]
fn splice_at_exact_end_of_sequence_appends() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 3, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9), // current; node_index + 1 == len (4)
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
        ],
        "when node_index + 1 == len, splice appends at the end"
    );
}

// ── 9 — `tiers_back: 2` regresses from tier 3 to tier 1, splices tier-1 ────-

#[test]
fn tiers_back_two_regresses_two_tiers_and_splices_target_tier() {
    let mut app = build_apply_app();
    install_config(&mut app, 2);
    install_pending(&mut app);
    install_outcome(&mut app, 5, 3, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0), // 0
            na(NodeType::Boss, 0, 1.0),   // 1
            na(NodeType::Active, 1, 0.9), // 2
            na(NodeType::Boss, 1, 0.9),   // 3
            na(NodeType::Active, 2, 0.8), // 4
            na(NodeType::Boss, 2, 0.8),   // 5 ← current
            na(NodeType::Active, 3, 0.7), // 6
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Active, 2, 0.8),
            (NodeType::Boss, 2, 0.8),
            (NodeType::Active, 1, 0.9), // spliced tier-1 clone
            (NodeType::Boss, 1, 0.9),   // spliced tier-1 clone
            (NodeType::Active, 3, 0.7),
        ],
        "target tier = 3 - 2 = 1; tier-1 nodes splice at index 6 preserving timer_mult 0.9"
    );
    assert_eq!(read_outcome(&app), (5, 1, 0));
}

// ── 10 — saturating clamp: tier 0 with `tiers_back: 1` regresses to 0 ──────-

#[test]
fn saturating_clamp_tier_zero_with_tiers_back_one_stays_at_tier_zero() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 0, 0, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0), // 0 ← current
            na(NodeType::Active, 0, 1.0), // 1
            na(NodeType::Boss, 0, 1.0),   // 2
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
        ],
        "target tier = 0.saturating_sub(1) = 0; all three tier-0 nodes splice at index 1"
    );
    let (_, tier, position) = read_outcome(&app);
    assert_eq!(tier, 0, "saturating clamp must NOT underflow to u32::MAX");
    assert_eq!(position, 0);
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
    );
}

// ── 11 — saturating clamp: tier 2 with `tiers_back: u32::MAX` → tier 0 ─────-

#[test]
fn saturating_clamp_tier_two_with_max_tiers_back_regresses_to_zero() {
    let mut app = build_apply_app();
    install_config(&mut app, u32::MAX);
    install_pending(&mut app);
    install_outcome(&mut app, 4, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
            na(NodeType::Boss, 2, 0.8), // 4 ← current
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Boss, 2, 0.8),
            (NodeType::Active, 0, 1.0), // spliced tier-0
            (NodeType::Boss, 0, 1.0),   // spliced tier-0
        ],
        "target tier = 2.saturating_sub(u32::MAX) = 0; tier-0 pair appended at end"
    );
    assert_eq!(read_outcome(&app), (4, 0, 0));
}

// ── 12 — target tier empty: no splice, but tier/position/pending updated ───-

#[test]
fn empty_target_tier_still_rewinds_tier_and_clears_pending() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 0, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 1, 0.9), // 0 ← current
            na(NodeType::Boss, 1, 0.9),   // 1
        ],
    );

    app.update();

    let seq = read_sequence(&app);
    assert_eq!(
        seq.len(),
        2,
        "no tier-0 nodes to clone — sequence length unchanged"
    );
    assert_eq!(
        seq,
        vec![(NodeType::Active, 1, 0.9), (NodeType::Boss, 1, 0.9),],
        "sequence unchanged by value when target tier has no assignments"
    );
    assert_eq!(
        read_outcome(&app),
        (0, 0, 0),
        "tier rewound and position reset even when splice is empty"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending removed (one-shot) even when splice is empty"
    );
}

// ── 13 — `tiers_back: 0` — no regression, but pending removed (one-shot) ───-

#[test]
fn tiers_back_zero_splices_current_tier_and_removes_pending() {
    let mut app = build_apply_app();
    install_config(&mut app, 0);
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0), // 0
            na(NodeType::Boss, 0, 1.0),   // 1
            na(NodeType::Boss, 1, 0.9),   // 2 ← current
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
        ],
        "target tier = 1.saturating_sub(0) = 1; tier-1 node at index 2 spliced at index 3 (end)"
    );
    let (_, tier, position) = read_outcome(&app);
    assert_eq!(tier, 1, "tier unchanged (target == current tier)");
    assert_eq!(position, 0, "position_in_tier reset to 0");
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "one-shot: pending removed even with tiers_back: 0"
    );
}

// ── 14 — `node_index` never modified ────────────────────────────────────────

#[test]
fn node_index_is_never_modified_by_apply() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 7, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
            na(NodeType::Active, 2, 0.8),
            na(NodeType::Boss, 2, 0.8),
        ],
    );

    app.update();

    let (node_index, ..) = read_outcome(&app);
    assert_eq!(
        node_index, 7,
        "node_index must be byte-for-byte unchanged by apply_tier_regression"
    );
}

// ── 15 — `result` and `cleared_this_frame` unchanged ───────────────────────-

#[test]
fn result_and_cleared_this_frame_unchanged_by_apply() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome_full(
        &mut app,
        NodeOutcome {
            node_index:         2,
            result:             NodeResult::Won,
            cleared_this_frame: true,
            tier:               1,
            position_in_tier:   0,
        },
    );
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Boss, 1, 0.9), // current
        ],
    );

    app.update();

    let outcome = app.world().resource::<NodeOutcome>();
    assert_eq!(outcome.result, NodeResult::Won, "result must be untouched");
    assert!(
        outcome.cleared_this_frame,
        "cleared_this_frame must be untouched"
    );
    assert_eq!(outcome.tier, 0, "tier rewound");
    assert_eq!(outcome.position_in_tier, 0, "position reset");
}

// ── Group B′ — one-shot and no-op guards ────────────────────────────────────

// ── 16 — second tick after splice is a no-op ───────────────────────────────-

#[test]
fn second_tick_after_splice_is_a_no_op() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    // Tick 1 — primary splice.
    app.update();
    let post_first_len = app.world().resource::<NodeSequence>().assignments.len();
    let post_first_seq = read_sequence(&app);
    let post_first_outcome = read_outcome(&app);

    // Tick 2 — must be a no-op: pending was removed on tick 1.
    app.update();

    assert_eq!(
        app.world().resource::<NodeSequence>().assignments.len(),
        post_first_len,
        "second tick must not alter sequence length"
    );
    assert_eq!(
        read_sequence(&app),
        post_first_seq,
        "second tick must not alter sequence contents"
    );
    assert_eq!(
        read_outcome(&app),
        post_first_outcome,
        "second tick must not alter outcome fields"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending remains absent after second tick"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── 16 edge — three total ticks all safe ───────────────────────────────────-

#[test]
fn three_ticks_after_splice_remain_stable() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    app.update();
    let post_first_seq = read_sequence(&app);
    app.update();
    app.update();

    assert_eq!(
        read_sequence(&app),
        post_first_seq,
        "three total ticks must leave the post-first-tick sequence intact"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending stays absent indefinitely after the one-shot splice"
    );
}

// ── 17 — missing config with pending present: no-op, pending removed ───────-

#[test]
fn missing_config_with_pending_present_is_no_op_and_removes_pending() {
    let mut app = build_apply_app();
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
        ],
        "sequence must be unchanged when config is absent"
    );
    assert_eq!(
        read_outcome(&app),
        (2, 1, 0),
        "outcome must be unchanged when config is absent"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending must be removed (pending-present branch clears it before config check)"
    );
}

// ── 17 edge — both config and pending absent: short-circuit, both absent ───-

#[test]
fn missing_config_and_pending_short_circuits_without_mutations() {
    let mut app = build_apply_app();
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
        ],
        "sequence unchanged when both config and pending absent"
    );
    assert_eq!(read_outcome(&app), (2, 1, 0));
    assert!(
        app.world().get_resource::<TierRegressionConfig>().is_none(),
        "config stays absent (no side-effect insert)"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending stays absent"
    );
}

// ── 18 — missing NodeSequence: no-op, pending removed ──────────────────────-

#[test]
fn missing_node_sequence_does_not_panic_and_pending_removed() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 0);
    // Intentionally no NodeSequence.

    app.update();

    // Reaching here without panic is half the assertion.
    let (node_index, tier, position) = read_outcome(&app);
    assert_eq!(node_index, 2);
    assert_eq!(tier, 1, "tier unchanged when NodeSequence absent");
    assert_eq!(position, 0);
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending removed despite missing NodeSequence"
    );
}

// ── 19 — missing NodeOutcome: no-op, pending removed ───────────────────────-

#[test]
fn missing_node_outcome_does_not_panic_and_pending_removed() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_sequence_with_tiers(
        &mut app,
        vec![na(NodeType::Active, 0, 1.0), na(NodeType::Boss, 0, 1.0)],
    );
    // Intentionally no NodeOutcome.

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![(NodeType::Active, 0, 1.0), (NodeType::Boss, 0, 1.0),],
        "sequence unchanged when NodeOutcome absent"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "pending removed despite missing NodeOutcome"
    );
}

// ── 20 — pending absent: system body is inert (config + sequence untouched) ─

#[test]
fn pending_absent_is_inert_even_with_config_and_sequence_present() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    // No install_pending().
    install_outcome(&mut app, 2, 1, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
        ],
    );

    app.update();

    assert_eq!(
        read_sequence(&app),
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
        ],
        "sequence byte-for-byte unchanged when pending absent"
    );
    assert_eq!(
        read_outcome(&app),
        (2, 1, 0),
        "outcome byte-for-byte unchanged when pending absent"
    );
    assert_eq!(
        *app.world()
            .get_resource::<TierRegressionConfig>()
            .expect("config preserved"),
        TierRegressionConfig { tiers_back: 1 },
    );
}

// ── Coverage — position_in_tier rewind from non-zero ───────────────────────-

/// Regression guard: `apply_tier_regression` must unconditionally reset
/// `outcome.position_in_tier` to 0 even when it started non-zero. Every
/// spec-listed behavior uses `position_in_tier: 0` as input, so this test
/// pins the reset directly.
#[test]
fn position_in_tier_resets_to_zero_even_when_starting_non_zero() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 2, 1, 3);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
        ],
    );

    app.update();

    let (_, tier, position_in_tier) = read_outcome(&app);
    assert_eq!(tier, 0, "tier rewound to target");
    assert_eq!(
        position_in_tier, 0,
        "position_in_tier must reset to 0 even when starting at 3"
    );
}

// ── Coverage — filter precision across three co-existing tiers ─────────────-

/// Regression guard: when the sequence simultaneously contains tier-0,
/// tier-1, and tier-2 nodes and `tiers_back: 1` with `outcome.tier == 2`,
/// only tier-1 assignments must be spliced — tier-0 and tier-2 nodes
/// must not leak into the splice.
#[test]
fn filter_splices_only_target_tier_when_three_tiers_coexist() {
    let mut app = build_apply_app();
    install_config(&mut app, 1);
    install_pending(&mut app);
    install_outcome(&mut app, 5, 2, 0);
    install_sequence_with_tiers(
        &mut app,
        vec![
            na(NodeType::Active, 0, 1.0),
            na(NodeType::Boss, 0, 1.0),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Active, 1, 0.9),
            na(NodeType::Boss, 1, 0.9),
            na(NodeType::Active, 2, 0.8),
            na(NodeType::Boss, 2, 0.8),
        ],
    );

    app.update();

    let seq = read_sequence(&app);
    assert_eq!(
        seq,
        vec![
            (NodeType::Active, 0, 1.0),
            (NodeType::Boss, 0, 1.0),
            (NodeType::Active, 1, 0.9),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Active, 2, 0.8),
            (NodeType::Active, 1, 0.9),
            (NodeType::Active, 1, 0.9),
            (NodeType::Boss, 1, 0.9),
            (NodeType::Boss, 2, 0.8),
        ],
        "only tier-1 nodes must be spliced at index 6 — tier-0 and tier-2 must not leak"
    );
}
