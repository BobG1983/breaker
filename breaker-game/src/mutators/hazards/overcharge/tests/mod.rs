//! Tests for the Overcharge hazard — coverage sweep.
//!
//! Groups mirror the test spec's A–G layout:
//! - A: `OverchargeConfig::per_kill_multiplier` formula (pure unit)
//! - B: `overcharge_count_kills` system
//! - C: `overcharge_reset_on_bump` system
//! - D: `overcharge_apply_speed` reconcile
//! - E: `activate` lifecycle
//! - F: `wire`-wired integration + gating
//! - G: Multi-bolt, Haste synergy, cleanup

mod helpers;

mod activate;
mod apply_speed;
mod count_kills;
mod formula;
mod reset_on_bump;
mod synergy;
mod wire;
