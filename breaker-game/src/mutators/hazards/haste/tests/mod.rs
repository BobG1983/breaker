//! Tests for the Haste hazard — coverage sweep.
//!
//! Groups mirror the test spec's A–G layout:
//! - A: `HasteConfig::multiplier` formula (pure unit)
//! - B: `haste_apply_speed` on bolts without an `EffectStack`
//! - C: `haste_apply_speed` reconciliation on existing stacks
//! - D: `wire` — scheduling, run-condition gates
//! - E: `activate` lifecycle
//! - F: No-config guard
//! - G: Cross-hazard / chip synergy pinning

mod helpers;

mod activate;
mod apply_fresh;
mod formula;
mod no_config;
mod reconcile;
mod synergy;
mod wire;
