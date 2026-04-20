//! Siphon protocol — kill-streak farms time back onto the node timer.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/siphon.md`.
//!
//! Owns the `SiphonConfig` resource (per-run tuning), `SiphonStreak` resource
//! (per-run streak tracker, cleared on node exit), the three runtime systems
//! (`siphon_tick_streak`, `siphon_on_cell_destroyed`, `siphon_cleanup_node`),
//! and the `SIPHON_SENTINEL` tag reserved for future heal/debug plumbing.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{SiphonStreak, activate, register};
