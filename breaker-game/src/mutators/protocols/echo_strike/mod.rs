//! Echo Strike protocol — Perfect-bump primed echo network.
//!
//! Design doc: `docs/design/protocols/echo_strike.md`.
//!
//! Owns the `EchoStrikeConfig` resource (per-run tuning), the `EchoNetwork`
//! and `EchoPrimed` per-bolt components, the four runtime systems
//! (`echo_strike_on_bump`, `echo_strike_emit_siblings`,
//! `echo_strike_cleanup_destroyed_echoes`, `echo_strike_cleanup_node`), and
//! the `activate` / `wire` dispatch entry points.

pub mod system;

#[cfg(test)]
mod tests;

pub use system::{EchoNetwork, EchoPrimed};
pub(crate) use system::{activate, wire};
