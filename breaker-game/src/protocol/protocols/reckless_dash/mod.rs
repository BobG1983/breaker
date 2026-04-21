//! Reckless Dash protocol — risky-zone damage boost with double-penalty
//! on bolt loss.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/reckless_dash.md`.
//!
//! Owns the `RecklessDashConfig` resource (per-run tuning), the
//! `RiskyDamageBoost` per-bolt component, the `RecklessDashDoubledBolts`
//! per-node tracking resource, the four runtime systems
//! (`reckless_dash_on_bump`, `reckless_dash_amplify_damage`,
//! `reckless_dash_double_penalty`, `reckless_dash_cleanup_node`), the
//! `RECKLESS_DASH_SENTINEL` source-chip tag, and the `activate` / `register`
//! dispatch entry points.

pub mod system;

#[cfg(test)]
mod tests;

pub use system::{RecklessDashDoubledBolts, RiskyDamageBoost};
pub(crate) use system::{activate, register};
