//! Section C — `establish_tether_links` system (Behaviours 20–35).
//!
//! Fires `OnEnter(NodeState::Playing)` behind `hazard_active(Tether)`.
//! Pins mutual-exclusion matching, adjacency-radius inclusivity, determinism
//! under shared `GameRng`, harness-safety when resources are absent, and
//! run-condition / filter behavior.
//!
//! Tests are grouped by behavior topic across sibling files:
//!
//! - [`coverage_counts`] — pair-count expectations across hazard stacks (B20–23)
//! - [`bidirectional`]   — link mutuality (B24)
//! - [`adjacency`]       — adjacency-radius inclusivity / exclusivity (B25–26)
//! - [`determinism`]     — same/different seed → same/different selection (B27–28)
//! - [`harness_safety`]  — missing resources + run-condition gating (B29–31)
//! - [`filtering`]       — Dead/Invulnerable filters + mutual exclusion (B32–35, W7)

mod link_helpers;

mod adjacency;
mod bidirectional;
mod coverage_counts;
mod determinism;
mod filtering;
mod harness_safety;
