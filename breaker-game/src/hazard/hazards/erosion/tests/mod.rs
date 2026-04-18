//! Tests for the Erosion hazard — coverage sweep.
//!
//! Groups mirror the test spec's A–F layout:
//! - A: `erosion_shrink` formula and clamp
//! - B: `erosion_restore` bump-grade matrix
//! - C: `erosion_apply_width` `EffectStack` reconciliation
//! - D: register — scheduling, run-condition gates, intra-frame ordering
//! - E: activate lifecycle
//! - F: cross-hazard / cross-source synergy pinning

mod helpers;

mod activate;
mod apply_width;
mod register;
mod restore;
mod shrink;
mod synergy;
