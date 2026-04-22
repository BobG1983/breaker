//! Tier Regression protocol — one-shot node-sequence splice that replays
//! the previous tier's pre-generated nodes.
//!
//! Design doc: `docs/design/protocols/tier_regression.md`.
//!
//! Owns the `TierRegressionConfig` / `TierRegressionPending` resources, the
//! `activate` / `register` dispatch entry points, and two runtime systems:
//! `snapshot_pre_advance_state` (runs `OnEnter(RunState::Node)` ordered
//! `.before(NodeSystems::AdvanceNode)`) captures the pre-advance
//! `NodeOutcome.tier` / `node_index` into `TierRegressionPending`, and
//! `apply_tier_regression` (runs `OnEnter(RunState::Node)` ordered
//! `.after(NodeSystems::AdvanceNode)`) splices cloned `NodeAssignment`s from
//! the target tier into `NodeSequence.assignments` at the pre-advance
//! `node_index + 1`, rewinds `NodeOutcome.tier`, and resets
//! `NodeOutcome.position_in_tier`. The `.after` ordering is load-bearing:
//! when the just-completed node is a Boss, `advance_node` would otherwise
//! carry the tier back up immediately after the rewind. By running last,
//! `apply_tier_regression` is the final authority over `outcome.tier` and
//! `outcome.position_in_tier`. `TierRegressionPending` is always removed
//! after `apply_tier_regression` runs (one-shot).

use bevy::prelude::*;

use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
    state::run::{
        node::NodeSystems,
        resources::{NodeAssignment, NodeOutcome, NodeSequence},
    },
};

// ── TierRegressionConfig ────────────────────────────────────────────────────

/// Per-run `TierRegression` tuning extracted from
/// `ProtocolTuning::TierRegression` at activation time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct TierRegressionConfig {
    /// How many tiers to regress on the next node transition.
    pub(crate) tiers_back: u32,
}

// ── TierRegressionPending ───────────────────────────────────────────────────

/// Inserted by `activate` and consumed by `apply_tier_regression` on the
/// next `OnEnter(RunState::Node)` transition. Absence of this marker is the
/// one-shot signal that regression has already occurred.
///
/// `activation_tier` and `activation_node_index` are snapshots of
/// `NodeOutcome.tier` and `NodeOutcome.node_index` captured by
/// `snapshot_pre_advance_state` BEFORE `advance_node` runs on the
/// `OnEnter(RunState::Node)` edge that triggers the splice. They are
/// `None` when the snapshot system has not yet run (e.g. at activation
/// time, or when `apply_tier_regression` is exercised directly on the
/// `Update` schedule in Group B unit tests). `apply_tier_regression`
/// falls back to the current `NodeOutcome` values when they are `None`.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TierRegressionPending {
    /// Pre-advance `NodeOutcome.tier` snapshot. Filled by
    /// `snapshot_pre_advance_state`; `None` before that system runs.
    pub(super) activation_tier:       Option<u32>,
    /// Pre-advance `NodeOutcome.node_index` snapshot. Filled by
    /// `snapshot_pre_advance_state`; `None` before that system runs.
    pub(super) activation_node_index: Option<u32>,
}

// ── activate ────────────────────────────────────────────────────────────────

/// Activation entry. Inserts `TierRegressionConfig` + `TierRegressionPending`
/// on matching `ProtocolTuning::TierRegression`; no-op on any other variant.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::TierRegression { tiers_back } = *tuning else {
        warn!("tier_regression::activate called with non-TierRegression tuning");
        return;
    };
    commands.insert_resource(TierRegressionConfig { tiers_back });
    commands.insert_resource(TierRegressionPending::default());
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers `snapshot_pre_advance_state` on `OnEnter(RunState::Node)`
/// ordered `.before(NodeSystems::AdvanceNode)` and `apply_tier_regression`
/// on the same edge ordered `.after(NodeSystems::AdvanceNode)`. Both are
/// gated by `protocol_active(ProtocolKind::TierRegression)` AND
/// `resource_exists::<TierRegressionPending>`.
///
/// The ordering is load-bearing. `snapshot_pre_advance_state` captures the
/// pre-advance `NodeOutcome.tier` / `node_index` before `advance_node`
/// mutates them (critical for the Boss-boundary case where `advance_node`
/// would otherwise bump `tier` and mask the real activation tier). Then
/// `advance_node` increments `node_index` and updates `tier` /
/// `position_in_tier` from the just-completed node's type. Finally
/// `apply_tier_regression` rewinds `tier` from the captured snapshot,
/// resets `position_in_tier`, and splices the target tier's assignments.
/// This makes `apply_tier_regression` the final authority over
/// `outcome.tier` and `outcome.position_in_tier`.
///
/// No `init_resource` — both resources are activation-driven, not standing
/// per-run defaults.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        OnEnter(RunState::Node),
        (
            snapshot_pre_advance_state
                .before(NodeSystems::AdvanceNode)
                .run_if(protocol_active(ProtocolKind::TierRegression))
                .run_if(resource_exists::<TierRegressionPending>),
            apply_tier_regression
                .after(NodeSystems::AdvanceNode)
                .run_if(protocol_active(ProtocolKind::TierRegression))
                .run_if(resource_exists::<TierRegressionPending>),
        ),
    );
}

// ── snapshot_pre_advance_state ──────────────────────────────────────────────

/// Pre-advance snapshot. Runs on `OnEnter(RunState::Node)` ordered
/// `.before(NodeSystems::AdvanceNode)` whenever `TierRegressionPending` is
/// present and `ProtocolKind::TierRegression` is in `ActiveProtocols`.
///
/// Captures `NodeOutcome.tier` and `NodeOutcome.node_index` into the
/// pending resource's `activation_tier` / `activation_node_index` fields
/// so `apply_tier_regression` (which runs AFTER `advance_node`) can
/// compute the rewind target from the pre-advance tier rather than the
/// potentially-bumped post-advance tier. Only fills `None` fields —
/// subsequent entries that hit the gate with an already-captured
/// activation preserve the original snapshot. When `NodeOutcome` is
/// absent, both fields remain `None` and `apply_tier_regression` falls
/// back to the current `NodeOutcome` at execution time.
pub(crate) fn snapshot_pre_advance_state(
    mut pending: ResMut<TierRegressionPending>,
    outcome: Option<Res<NodeOutcome>>,
) {
    // `activation_tier` is the authoritative "already snapshotted"
    // sentinel; both fields are always written together below, so
    // checking one is sufficient.
    if pending.activation_tier.is_some() {
        return;
    }
    if let Some(outcome) = outcome {
        pending.activation_tier = Some(outcome.tier);
        pending.activation_node_index = Some(outcome.node_index);
    }
}

// ── apply_tier_regression ───────────────────────────────────────────────────

/// One-shot splice system. Runs on `OnEnter(RunState::Node)` ordered
/// `.after(NodeSystems::AdvanceNode)` whenever `TierRegressionPending` is
/// present and `ProtocolKind::TierRegression` is in `ActiveProtocols`.
/// Removes the pending marker after execution regardless of splice
/// outcome.
///
/// Resources are taken as `Option<...>` because the `run_if` chain only
/// gates on `TierRegressionPending`; `NodeOutcome`, `NodeSequence`, and
/// `TierRegressionConfig` are expected but not required — the body
/// degrades gracefully and always clears the pending marker.
///
/// Uses `pending.activation_tier` / `pending.activation_node_index` (when
/// set by `snapshot_pre_advance_state`) as the pre-advance basis for the
/// rewind and splice position. When those fields are `None` (Group B unit
/// tests that exercise the body directly on `Update` without the snapshot
/// system), falls back to the current `NodeOutcome` values — which in
/// Group B already match pre-advance semantics because `advance_node`
/// does not run in that schedule.
pub(crate) fn apply_tier_regression(
    pending: Option<Res<TierRegressionPending>>,
    config: Option<Res<TierRegressionConfig>>,
    outcome: Option<ResMut<NodeOutcome>>,
    sequence: Option<ResMut<NodeSequence>>,
    mut commands: Commands,
) {
    // 1. No-pending defensive guard — silent early-return.
    let Some(pending) = pending else {
        return;
    };

    // 2. Short-circuit on any missing optional resource. Still remove the
    //    pending marker to prevent a stuck-pending state on the next
    //    `OnEnter(RunState::Node)`.
    let (Some(config), Some(mut outcome), Some(mut sequence)) = (config, outcome, sequence) else {
        warn!(
            "tier_regression::apply_tier_regression pending but resources missing — removing marker"
        );
        commands.remove_resource::<TierRegressionPending>();
        return;
    };

    // 3. Pre-advance values come from the snapshot when available; fall
    //    back to current outcome when the snapshot system has not run
    //    (Group B unit tests on `Update`). In Group C the snapshot has
    //    fired before `advance_node`, so these are the values BEFORE any
    //    Boss-induced tier bump.
    let pre_advance_tier = pending.activation_tier.unwrap_or(outcome.tier);
    let pre_advance_node_index = pending.activation_node_index.unwrap_or(outcome.node_index);

    // 4. Compute target tier with saturating clamp (handles tier-0 and
    //    u32::MAX edge cases without underflow).
    let target_tier: u32 = pre_advance_tier.saturating_sub(config.tiers_back);

    // 5. Collect a clone of every assignment whose tier matches the target.
    let regressed: Vec<NodeAssignment> = sequence
        .assignments
        .iter()
        .filter(|a| a.tier_index == target_tier)
        .cloned()
        .collect();

    // 6. Compute splice index from the pre-advance `node_index + 1`,
    //    clamped to end-of-sequence.
    let splice_at = ((pre_advance_node_index as usize) + 1).min(sequence.assignments.len());

    // 7. Splice — only if we actually have assignments to insert.
    if regressed.is_empty() {
        warn!(
            "tier_regression::apply_tier_regression found no assignments for target tier {target_tier} — skipping splice"
        );
    } else {
        drop(sequence.assignments.splice(splice_at..splice_at, regressed));
    }

    // 8. Rewind outcome unconditionally — tier + position only; do NOT
    //    touch `node_index`, `result`, or `cleared_this_frame`.
    outcome.tier = target_tier;
    outcome.position_in_tier = 0;

    // 9. One-shot: remove the pending marker unconditionally so the next
    //    `OnEnter(RunState::Node)` is gated off by
    //    `resource_exists::<TierRegressionPending>`.
    commands.remove_resource::<TierRegressionPending>();
}
