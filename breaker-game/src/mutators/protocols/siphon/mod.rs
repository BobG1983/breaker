//! Siphon protocol — kill-streak farms time back onto the node timer.
//!
//! Design doc: `docs/design/protocols/siphon.md`.
//!
//! Owns the `SiphonConfig` resource (per-run tuning), `SiphonStreak` resource
//! (per-run streak tracker, cleared on node exit), and the three runtime
//! systems (`siphon_tick_streak`, `siphon_on_cell_destroyed`,
//! `siphon_cleanup_node`).

pub mod system;

#[cfg(test)]
mod tests;

pub use system::SiphonStreak;
pub(crate) use system::{activate, register};
