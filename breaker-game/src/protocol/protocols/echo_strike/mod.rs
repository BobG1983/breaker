//! Echo Strike protocol — Perfect-bump primed echo network.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/echo_strike.md`.
//!
//! Owns the `EchoStrikeConfig` resource (per-run tuning), the `EchoNetwork`
//! and `EchoPrimed` per-bolt components, the four runtime systems
//! (`echo_strike_on_bump`, `echo_strike_on_impact`,
//! `echo_strike_cleanup_destroyed_echoes`, `echo_strike_cleanup_node`), the
//! `ECHO_STRIKE_SENTINEL` source-chip tag, and the `activate` / `register`
//! dispatch entry points.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
