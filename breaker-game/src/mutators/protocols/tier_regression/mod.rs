//! Tier Regression protocol — one-shot node-sequence splice that replays
//! the previous tier's pre-generated nodes.
//!
//! Design doc: `docs/design/protocols/tier_regression.md`.
//!
//! Owns the `TierRegressionConfig` / `TierRegressionPending` resources and
//! two runtime systems registered on `OnEnter(RunState::Node)`:
//! `snapshot_pre_advance_state` runs `.before(NodeSystems::AdvanceNode)` and
//! captures the pre-advance `NodeOutcome.tier` / `node_index` into
//! `TierRegressionPending`; `apply_tier_regression` runs
//! `.after(NodeSystems::AdvanceNode)`, splices cloned assignments from the
//! target tier into `NodeSequence.assignments` at the pre-advance
//! `node_index + 1`, rewinds `NodeOutcome.tier`, resets
//! `NodeOutcome.position_in_tier`, and removes `TierRegressionPending`
//! (one-shot). The `.after` ordering makes `apply_tier_regression` the
//! final authority over `outcome.tier` / `outcome.position_in_tier`,
//! preventing the Boss-boundary bug where `advance_node`'s tier increment
//! would silently cancel the rewind.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, wire};
